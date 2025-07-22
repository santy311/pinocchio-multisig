use pinocchio::{
    program_error::ProgramError,
    pubkey::{self, Pubkey},
};

use crate::{error::MultisigError, state::utils::DataLen};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, shank::ShankType)]
pub struct Multisig {
    pub creator: Pubkey,
    pub threshold: u8,
    pub num_members: u8,
    pub bump: u8,
}

impl DataLen for Multisig {
    const LEN: usize = 32 + 1 + 1 + 1;
}

impl Multisig {
    pub const SEED: &'static str = "multisig";

    pub fn new(creator: Pubkey, threshold: u8, num_members: u8, bump: u8) -> Self {
        Self {
            creator,
            threshold,
            num_members,
            bump,
        }
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

        let mapping = unsafe { *(bytes.as_ptr() as *const Self) };
        Ok(mapping)
    }

    pub fn to_bytes(&self) -> [u8; Self::LEN] {
        let mut bytes = [0u8; Self::LEN];

        unsafe {
            core::ptr::copy_nonoverlapping(
                self as *const Self as *const u8,
                bytes.as_mut_ptr(),
                Self::LEN,
            );
        }
        bytes
    }
}
