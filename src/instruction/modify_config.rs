use pinocchio::{account_info::AccountInfo, msg, program_error::ProgramError, ProgramResult};

use crate::state::{multisig::Multisig, DataLen};

pub fn process_modify_config_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    msg!("Processing modify config instruction");
    let [payer_acc, multisig_acc, _remaining_accounts @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !payer_acc.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if multisig_acc.data_is_empty() {
        return Err(ProgramError::InvalidAccountData);
    }

    let ix_data = ModifyConfigData::from_bytes(data);

    // Validate the PDA
    Multisig::validate_pda(ix_data.bump, multisig_acc.key(), ix_data.multisig_id)?;

    let multisig_data = unsafe { multisig_acc.borrow_mut_data_unchecked() };
    let mut multisig = Multisig::from_bytes(&mut multisig_data[..Multisig::LEN])?;

    multisig.threshold = ix_data.threshold;
    multisig.max_expiry_duration = ix_data.max_expiry_duration;
    multisig.veto_threshold = ix_data.veto_threshold;

    multisig_data[..Multisig::LEN].copy_from_slice(&multisig.to_bytes());

    Ok(())
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, shank::ShankType)]
pub struct ModifyConfigData {
    pub threshold: u8,
    pub max_expiry_duration: u32,
    pub veto_threshold: u8,
    pub bump: u8,
    pub multisig_id: u64,
}

impl DataLen for ModifyConfigData {
    const LEN: usize = 1 + 4 + 1 + 1 + 8;
}

impl ModifyConfigData {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let threshold = bytes[0];
        let max_expiry_duration = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
        let veto_threshold = bytes[5];
        let bump = bytes[6];
        let multisig_id = u64::from_le_bytes([
            bytes[7], bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14],
        ]);
        Self {
            threshold,
            max_expiry_duration,
            veto_threshold,
            bump,
            multisig_id,
        }
    }

    pub fn to_bytes(&self) -> [u8; Self::LEN] {
        let mut bytes = [0u8; Self::LEN];
        bytes[0] = self.threshold;
        bytes[1..5].copy_from_slice(&self.max_expiry_duration.to_le_bytes());
        bytes[5] = self.veto_threshold;
        bytes[6] = self.bump;
        bytes[7..15].copy_from_slice(&self.multisig_id.to_le_bytes());
        bytes
    }
}
