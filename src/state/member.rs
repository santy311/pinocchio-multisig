use pinocchio::program_error::ProgramError;

use crate::state::utils::DataLen;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, shank::ShankType)]
pub struct Member {
    pub pubkey: [u8; 32],
    pub member_id: u8,
    pub role: u8, // 0: member, 1: admin
}

impl DataLen for Member {
    const LEN: usize = 32 + 1 + 1;
}

impl Member {
    pub fn new(pubkey: [u8; 32], role: u8) -> Self {
        Self {
            pubkey,
            member_id: 0,
            role,
        }
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
