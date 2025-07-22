use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    sysvars::rent::Rent,
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

use crate::state::{load_ix_data, multisig::Multisig, DataLen, Member};

pub fn process_init_multisig_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [payer_acc, multisig_acc, sysvar_rent_acc, _remaining_accounts @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !payer_acc.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !multisig_acc.data_is_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let rent = Rent::from_account_info(sysvar_rent_acc)?;

    let (ix_data_bytes, members_bytes) = data.split_at(InitMultisigData::LEN);
    let ix_data = InitMultisigData::from_bytes(ix_data_bytes);

    let pda_bump_bytes = [ix_data.bump];

    // Validate the PDA
    Multisig::validate_pda(ix_data.bump, multisig_acc.key(), payer_acc.key())?;

    // Signer seeds
    let signer_seeds = [
        Seed::from(Multisig::SEED.as_bytes()),
        Seed::from(payer_acc.key().as_ref()),
        Seed::from(&pda_bump_bytes[..]),
    ];
    let signers = [Signer::from(&signer_seeds[..])];

    let space = Multisig::LEN + Member::LEN * ix_data.num_members as usize;

    let multisig = Multisig::new(
        *payer_acc.key(),
        ix_data.threshold,
        ix_data.num_members,
        ix_data.bump,
    );

    // Create the account
    CreateAccount {
        from: payer_acc,
        to: multisig_acc,
        space: space as u64,
        owner: &crate::ID,
        lamports: rent.minimum_balance(space),
    }
    .invoke_signed(&signers)?;

    let multisig_data = unsafe { multisig_acc.borrow_mut_data_unchecked() };
    multisig_data[..Multisig::LEN].copy_from_slice(&multisig.to_bytes());

    for (i, member) in members_bytes.chunks(Member::LEN).enumerate() {
        let member_data = unsafe { load_ix_data::<Member>(member)? };
        let start_offset = Multisig::LEN + i * Member::LEN;
        let end_offset = start_offset + Member::LEN;
        let member_segment = &mut multisig_data[start_offset..end_offset];
        member_segment.copy_from_slice(&member_data.to_bytes());
        member_segment[32] = i as u8;
    }

    Ok(())
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, shank::ShankType)]
pub struct InitMultisigData {
    pub threshold: u8,
    pub num_members: u8,
    pub bump: u8,
}

impl DataLen for InitMultisigData {
    const LEN: usize = 1 + 1 + 1;
}

impl InitMultisigData {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let threshold = bytes[0];
        let num_members = bytes[1];
        let bump = bytes[2];
        Self {
            threshold,
            num_members,
            bump,
        }
    }

    pub fn to_bytes(&self) -> [u8; Self::LEN] {
        let mut bytes = [0u8; Self::LEN];
        bytes[0] = self.threshold;
        bytes[1] = self.num_members;
        bytes[2] = self.bump;
        bytes
    }
}
