use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    msg,
    program_error::ProgramError,
    sysvars::rent::Rent,
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

use crate::state::{multisig::Multisig, DataLen, Member, Vault};

pub fn process_init_multisig_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [payer_acc, multisig_acc, vault_acc, sysvar_rent_acc, _remaining_accounts @ ..] = accounts
    else {
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
    Multisig::validate_pda(ix_data.bump, multisig_acc.key(), ix_data.multisig_id)?;

    Vault::validate_pda(ix_data.vault_bump, vault_acc.key(), multisig_acc.key())?;

    // Signer seeds
    let multisig_id_bytes = ix_data.multisig_id.to_le_bytes();
    let signer_seeds = [
        Seed::from(Multisig::SEED.as_bytes()),
        Seed::from(&multisig_id_bytes[..]),
        Seed::from(&pda_bump_bytes[..]),
    ];
    let signers = [Signer::from(&signer_seeds[..])];

    let space = Multisig::LEN + Member::LEN * ix_data.num_members as usize;

    let multisig = Multisig::new(
        *payer_acc.key(),
        ix_data.threshold,
        ix_data.num_members,
        ix_data.max_expiry_duration,
        ix_data.veto_threshold,
        *vault_acc.key(),
        ix_data.multisig_id,
        ix_data.bump,
        ix_data.vault_bump,
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
        let member_data = Member::from_bytes(member)?;
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
    pub max_expiry_duration: u32,
    pub veto_threshold: u8,
    pub multisig_id: u64,
    pub bump: u8,
    pub vault_bump: u8,
}

impl DataLen for InitMultisigData {
    const LEN: usize = 1 + 1 + 4 + 1 + 8 + 1 + 1;
}

impl InitMultisigData {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        assert!(
            bytes.len() >= Self::LEN,
            "Not enough bytes to deserialize InitMultisigData"
        );
        let threshold = bytes[0];
        let num_members = bytes[1];
        let max_expiry_duration = u32::from_le_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]);
        let veto_threshold = bytes[6];
        let multisig_id = u64::from_le_bytes([
            bytes[7], bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14],
        ]);
        let bump = bytes[15];
        let vault_bump = bytes[16];
        Self {
            threshold,
            num_members,
            max_expiry_duration,
            veto_threshold,
            multisig_id,
            bump,
            vault_bump,
        }
    }

    pub fn to_bytes(&self) -> [u8; Self::LEN] {
        let mut bytes = [0u8; Self::LEN];
        bytes[0] = self.threshold;
        bytes[1] = self.num_members;
        bytes[2..6].copy_from_slice(&self.max_expiry_duration.to_le_bytes());
        bytes[6] = self.veto_threshold;
        bytes[7..15].copy_from_slice(&self.multisig_id.to_le_bytes());
        bytes[15] = self.bump;
        bytes[16] = self.vault_bump;
        bytes
    }
}
