use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, sysvars::rent::Rent, ProgramResult,
};

use crate::{
    error::MultisigError,
    state::{check_admin_action, load_ix_data, multisig::Multisig, DataLen, Member},
};

pub fn process_remove_member_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
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

    let (ix_data_bytes, _members_bytes) = data.split_at(RemoveMemberData::LEN);
    let ix_data = RemoveMemberData::from_bytes(ix_data_bytes);

    // Validate the PDA
    Multisig::validate_pda(ix_data.bump, multisig_acc.key(), ix_data.multisig_id)?;

    check_admin_action(payer_acc.key(), unsafe {
        multisig_acc.borrow_data_unchecked()
    })?;

    let space = multisig_acc.data_len() - Member::LEN;
    let rent_diff = multisig_acc.lamports() - rent.minimum_balance(space);

    if rent_diff > 0 {
        unsafe {
            *multisig_acc.borrow_mut_lamports_unchecked() = multisig_acc.lamports() - rent_diff;
            *payer_acc.borrow_mut_lamports_unchecked() = payer_acc.lamports() + rent_diff;
        }
    }

    let multisig_data = unsafe { multisig_acc.borrow_mut_data_unchecked() };
    let mut multisig = Multisig::from_bytes(&mut multisig_data[..Multisig::LEN])?;

    let old_num_members = multisig.num_members;

    let mut member_index = None;
    for (i, member) in multisig_data[Multisig::LEN..]
        .chunks(Member::LEN)
        .enumerate()
    {
        let member_data = unsafe { load_ix_data::<Member>(member)? };
        if member_data.member_id == ix_data.member_id {
            member_index = Some(i);
            break;
        }
    }

    if let Some(member_index) = member_index {
        let member_offset = Multisig::LEN + member_index * Member::LEN;

        // Copy the last member to the removed member's position
        let last_member_offset = Multisig::LEN + (old_num_members - 1) as usize * Member::LEN;

        // Read the last member data before modifying multisig_data
        let last_member_data = unsafe {
            load_ix_data::<Member>(
                &multisig_data[last_member_offset..last_member_offset + Member::LEN],
            )?
        };
        let last_member_bytes = last_member_data.to_bytes();

        // Now we can safely modify multisig_data
        multisig_data[member_offset..member_offset + Member::LEN]
            .copy_from_slice(&last_member_bytes);

        // Clear the last member's data
        for b in &mut multisig_data[last_member_offset..last_member_offset + Member::LEN] {
            *b = 0;
        }

        // Update the member count
        multisig.num_members = old_num_members - 1;
        multisig_data[..Multisig::LEN].copy_from_slice(&multisig.to_bytes());

        multisig_acc.resize(space as usize)?;
    } else {
        return Err(MultisigError::MemberNotFound.into());
    }

    Ok(())
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, shank::ShankType)]
pub struct RemoveMemberData {
    pub member_id: u8,
    pub bump: u8,
    pub multisig_id: u64,
}

impl DataLen for RemoveMemberData {
    const LEN: usize = 1 + 1 + 8;
}

impl RemoveMemberData {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let member_id = bytes[0];
        let bump = bytes[1];
        let multisig_id = u64::from_le_bytes([
            bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8], bytes[9],
        ]);
        Self {
            member_id,
            bump,
            multisig_id,
        }
    }

    pub fn to_bytes(&self) -> [u8; Self::LEN] {
        let mut bytes = [0u8; Self::LEN];
        bytes[0] = self.member_id;
        bytes[1] = self.bump;
        bytes[2..10].copy_from_slice(&self.multisig_id.to_le_bytes());
        bytes
    }
}
