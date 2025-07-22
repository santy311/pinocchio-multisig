use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    msg,
    program_error::ProgramError,
    sysvars::rent::Rent,
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;
use shank::ShankType;

use crate::state::{get_admin_index_if_exists, load_ix_data, utils::DataLen, Multisig, Proposal};

pub fn process_create_proposal_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [payer_acc, multisig_acc, proposal_acc, sysvar_rent_acc, _remaining_accounts @ ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !payer_acc.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if multisig_acc.data_is_empty() {
        return Err(ProgramError::InvalidAccountData);
    }

    if !proposal_acc.data_is_empty() || proposal_acc.lamports() != 0 {
        msg!("Proposal account already exists");
        return Err(ProgramError::InvalidAccountData);
    }

    let rent = Rent::from_account_info(sysvar_rent_acc)?;

    let (ix_data_bytes, _members_bytes) = data.split_at(CreateProposalData::LEN);
    let ix_data = unsafe { load_ix_data::<CreateProposalData>(ix_data_bytes)? };

    let pda_bump_bytes = [ix_data.proposal_bump];

    let multisig_data = unsafe { multisig_acc.borrow_mut_data_unchecked() };
    let mut multisig = Multisig::from_bytes(&multisig_data[..Multisig::LEN])?;

    // Validate the PDA
    Proposal::validate_pda(
        ix_data.proposal_bump,
        proposal_acc.key(),
        multisig.proposal_counter,
        multisig_acc.key(),
    )?;

    // Signer seeds
    let proposal_id_bytes = (multisig.proposal_counter).to_le_bytes();
    let multisig_signer_seeds = [
        Seed::from(Proposal::SEED.as_bytes()),
        Seed::from(multisig_acc.key().as_ref()),
        Seed::from(proposal_id_bytes.as_ref()),
        Seed::from(&pda_bump_bytes[..]),
    ];
    let multisig_signers = [Signer::from(&multisig_signer_seeds[..])];

    let admin_index = get_admin_index_if_exists(payer_acc.key(), unsafe {
        multisig_acc.borrow_data_unchecked()
    })?;

    let space = Proposal::LEN;

    CreateAccount {
        from: payer_acc,
        to: proposal_acc,
        space: space as u64,
        owner: &crate::ID,
        lamports: rent.minimum_balance(space),
    }
    .invoke_signed(&multisig_signers)?;

    let proposal = Proposal::new(
        *multisig_acc.key(),
        multisig.proposal_counter,
        admin_index,
        multisig.max_expiry_duration,
        ix_data.proposal_bump,
    )?;

    let proposal_data = unsafe { proposal_acc.borrow_mut_data_unchecked() };
    proposal_data[..Proposal::LEN].copy_from_slice(&proposal.to_bytes());

    multisig.proposal_counter += 1;
    multisig_data[..Multisig::LEN].copy_from_slice(&multisig.to_bytes());
    Ok(())
}

#[repr(C)]
#[derive(Debug, Clone, Copy, ShankType)]
pub struct CreateProposalData {
    pub multisig_bump: u8,
    pub proposal_bump: u8,
}

impl DataLen for CreateProposalData {
    const LEN: usize = 1 + 1;
}

impl CreateProposalData {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            multisig_bump: bytes[0],
            proposal_bump: bytes[1],
        }
    }

    pub fn to_bytes(&self) -> [u8; Self::LEN] {
        [self.multisig_bump, self.proposal_bump]
    }
}
