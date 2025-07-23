use pinocchio::{
    account_info::AccountInfo,
    msg,
    program_error::ProgramError,
    sysvars::{clock::Clock, Sysvar},
    ProgramResult,
};

use crate::{
    error::MultisigError,
    state::{get_member_index_if_exists, multisig::Multisig, DataLen, Proposal, ProposalStatus},
};

pub fn process_execute_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    msg!("Processing execute instruction");
    let [payer_acc, multisig_acc, proposal_acc, _remaining_accounts @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !payer_acc.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if multisig_acc.data_is_empty() {
        return Err(ProgramError::InvalidAccountData);
    }

    let ix_data = ExecuteData::from_bytes(data);

    // Validate the PDA
    Multisig::validate_pda(ix_data.bump, multisig_acc.key(), ix_data.multisig_id)?;
    Proposal::validate_pda(
        ix_data.proposal_bump,
        proposal_acc.key(),
        ix_data.proposal_id,
        multisig_acc.key(),
    )?;

    let multisig_data = unsafe { multisig_acc.borrow_mut_data_unchecked() };
    let multisig = Multisig::from_bytes(&mut multisig_data[..Multisig::LEN])?;

    let proposal_data = unsafe { proposal_acc.borrow_mut_data_unchecked() };
    let mut proposal = Proposal::from_bytes(&mut proposal_data[..Proposal::LEN])?;

    if proposal.status != ProposalStatus::Pending as u8 {
        return Err(MultisigError::InvalidProposalStatus.into());
    }

    if proposal.expiry < Clock::get()?.unix_timestamp as u64 {
        return Err(MultisigError::ProposalNotExpired.into());
    }

    // if payer is part of multisig
    get_member_index_if_exists(payer_acc.key(), multisig_data)?;

    if proposal.veto_votes >= multisig.veto_threshold {
        proposal.status = ProposalStatus::Vetoed as u8;
    } else if proposal.yes_votes >= multisig.threshold {
        proposal.status = ProposalStatus::Approved as u8;
    } else if proposal.no_votes > multisig.threshold {
        proposal.status = ProposalStatus::Rejected as u8;
    }

    unsafe {
        proposal_acc.borrow_mut_data_unchecked()[..Proposal::LEN]
            .copy_from_slice(&proposal.to_bytes());
    }

    Ok(())
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, shank::ShankType)]
pub struct ExecuteData {
    pub proposal_id: u64,
    pub multisig_id: u64,
    pub bump: u8,
    pub vault_bump: u8,
    pub proposal_bump: u8,
}

impl DataLen for ExecuteData {
    const LEN: usize = 8 + 8 + 1 + 1 + 1;
}

impl ExecuteData {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let proposal_id = u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        let multisig_id = u64::from_le_bytes([
            bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
        ]);
        let bump = bytes[16];
        let vault_bump = bytes[17];
        let proposal_bump = bytes[18];
        Self {
            proposal_id,
            multisig_id,
            bump,
            vault_bump,
            proposal_bump,
        }
    }

    pub fn to_bytes(&self) -> [u8; Self::LEN] {
        let mut bytes = [0u8; Self::LEN];
        bytes[0..8].copy_from_slice(&self.proposal_id.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.multisig_id.to_le_bytes());
        bytes[16] = self.bump;
        bytes[17] = self.vault_bump;
        bytes[18] = self.proposal_bump;
        bytes
    }
}
