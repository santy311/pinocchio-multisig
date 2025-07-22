use crate::error::MultisigError;

use super::utils::DataLen;
use pinocchio::{
    program_error::ProgramError,
    pubkey::{self, Pubkey},
};
use shank::ShankType;

#[repr(C)]
#[derive(Debug, Clone, Copy, ShankType)]
pub struct Vault {
    pub multisig: [u8; 32],
    pub vault_bump: u8,
}

impl DataLen for Vault {
    const LEN: usize = 32 + 1;
}

impl Vault {
    pub const SEED: &'static str = "vault";

    pub fn new(multisig: [u8; 32], vault_bump: u8) -> Result<Self, ProgramError> {
        Ok(Self {
            multisig,
            vault_bump,
        })
    }

    pub fn validate_pda(bump: u8, pda: &Pubkey, owner: &Pubkey) -> Result<(), ProgramError> {
        let seed_with_bump = &[Self::SEED.as_bytes(), owner, &[bump]];
        let derived = pubkey::create_program_address(seed_with_bump, &crate::ID)?;
        if derived != *pda {
            return Err(MultisigError::PdaMismatch.into());
        }
        Ok(())
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ProgramError> {
        if bytes.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        let multisig = bytes[0..32].try_into().unwrap();
        let vault_bump = bytes[32];
        Ok(Self {
            multisig,
            vault_bump,
        })
    }

    pub fn to_bytes(&self) -> [u8; Self::LEN] {
        let mut bytes = [0u8; Self::LEN];
        bytes[0..32].copy_from_slice(&self.multisig);
        bytes[32] = self.vault_bump;
        bytes
    }
}
