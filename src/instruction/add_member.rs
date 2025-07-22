use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    sysvars::rent::Rent,
    ProgramResult,
};
use pinocchio_system::instructions::Transfer;

use crate::state::{check_admin_action, load_ix_data, multisig::Multisig, DataLen, Member};

pub fn process_add_member_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [payer_acc, multisig_acc, sysvar_rent_acc, _remaining_accounts @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !payer_acc.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if multisig_acc.data_is_empty() {
        return Err(ProgramError::InvalidAccountData);
    }

    let rent = Rent::from_account_info(sysvar_rent_acc)?;

    let (ix_data_bytes, members_bytes) = data.split_at(AddMemberData::LEN);
    let ix_data = unsafe { load_ix_data::<AddMemberData>(ix_data_bytes)? };

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

    let space = multisig_acc.data_len() + Member::LEN * ix_data.num_members as usize;

    let rent_diff = rent.minimum_balance(space) - multisig_acc.lamports();

    check_admin_action(payer_acc.key(), unsafe {
        multisig_acc.borrow_data_unchecked()
    })?;

    if rent_diff > 0 {
        Transfer {
            from: payer_acc,
            to: multisig_acc,
            lamports: rent_diff,
        }
        .invoke_signed(&signers)?;
    }

    multisig_acc.resize(space as usize)?;

    let multisig_data = unsafe { multisig_acc.borrow_mut_data_unchecked() };
    let mut multisig = Multisig::from_bytes(&mut multisig_data[..Multisig::LEN])?;
    let old_num_members = multisig.num_members;

    let last_member_offset = Multisig::LEN + old_num_members as usize * Member::LEN;
    for (i, member) in members_bytes.chunks(Member::LEN).enumerate() {
        let member_data = unsafe { load_ix_data::<Member>(member)? };
        let member_segment = &mut multisig_data
            [last_member_offset + i * Member::LEN..last_member_offset + (i + 1) * Member::LEN];
        let mut member_bytes = member_data.to_bytes();
        member_bytes[32] = old_num_members + i as u8;
        member_segment.copy_from_slice(&member_bytes);
    }

    multisig.num_members += ix_data.num_members;
    multisig_data[..Multisig::LEN].copy_from_slice(&multisig.to_bytes());

    Ok(())
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, shank::ShankType)]
pub struct AddMemberData {
    pub num_members: u8,
    pub bump: u8,
}

impl DataLen for AddMemberData {
    const LEN: usize = 1 + 1 + 1;
}

impl AddMemberData {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let num_members = bytes[1];
        let bump = bytes[2];
        Self { num_members, bump }
    }

    pub fn to_bytes(&self) -> [u8; Self::LEN] {
        let mut bytes = [0u8; Self::LEN];
        bytes[0] = self.num_members;
        bytes[1] = self.bump;
        bytes
    }
}
