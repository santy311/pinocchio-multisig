use pinocchio::{account_info::AccountInfo, msg, program_error::ProgramError, ProgramResult};

use crate::state::{load_ix_data, multisig::Multisig, DataLen};

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

    let ix_data = unsafe { load_ix_data::<ModifyConfigData>(data)? };

    // Validate the PDA
    Multisig::validate_pda(ix_data.bump, multisig_acc.key(), payer_acc.key())?;

    let multisig_data = unsafe { multisig_acc.borrow_mut_data_unchecked() };
    let mut multisig = Multisig::from_bytes(&mut multisig_data[..Multisig::LEN])?;

    multisig.threshold = ix_data.threshold;

    multisig_data[..Multisig::LEN].copy_from_slice(&multisig.to_bytes());

    Ok(())
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, shank::ShankType)]
pub struct ModifyConfigData {
    pub threshold: u8,
    pub bump: u8,
}

impl DataLen for ModifyConfigData {
    const LEN: usize = 1 + 1;
}

impl ModifyConfigData {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let threshold = bytes[0];
        let bump = bytes[1];
        Self { threshold, bump }
    }

    pub fn to_bytes(&self) -> [u8; Self::LEN] {
        let mut bytes = [0u8; Self::LEN];
        bytes[0] = self.threshold;
        bytes[1] = self.bump;
        bytes
    }
}
