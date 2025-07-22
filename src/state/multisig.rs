use pinocchio::{
    msg,
    program_error::ProgramError,
    pubkey::{self, Pubkey},
};

use crate::{
    error::MultisigError,
    state::{utils::DataLen, Member},
};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, shank::ShankType)]
pub struct Multisig {
    pub creator: [u8; 32],
    pub threshold: u8,
    pub num_members: u8,
    pub max_expiry_duration: u32,
    pub veto_threshold: u8,
    pub proposal_counter: u64,
    pub seed: u16,
    pub bump: u8,
}

impl DataLen for Multisig {
    const LEN: usize = 32 + 1 + 1 + 4 + 1 + 8 + 2 + 1;
}

impl Multisig {
    pub const SEED: &'static str = "multisig";

    pub fn new(
        creator: Pubkey,
        threshold: u8,
        num_members: u8,
        max_expiry_duration: u32,
        veto_threshold: u8,
        seed: u16,
        bump: u8,
    ) -> Self {
        Self {
            creator,
            threshold,
            num_members,
            max_expiry_duration,
            veto_threshold,
            proposal_counter: 0,
            seed,
            bump,
        }
    }

    pub fn validate_pda(bump: u8, pda: &Pubkey, owner: &Pubkey) -> Result<(), ProgramError> {
        let seed_with_bump = &[Self::SEED.as_bytes(), owner, &[bump]];
        let derived = pubkey::create_program_address(seed_with_bump, &crate::ID)?;
        if derived != *pda {
            msg!("PDA mismatch: {:?} != {:?}", derived, pda);
            return Err(MultisigError::PdaMismatch.into());
        }
        Ok(())
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ProgramError> {
        if bytes.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        let creator = unsafe { *(bytes.as_ptr() as *const [u8; 32]) };
        let threshold = bytes[32];
        let num_members = bytes[33];
        let max_expiry_duration =
            u32::from_le_bytes(unsafe { *(bytes.as_ptr().add(34) as *const [u8; 4]) });
        let veto_threshold = bytes[38];
        let proposal_counter =
            u64::from_le_bytes(unsafe { *(bytes.as_ptr().add(39) as *const [u8; 8]) });
        let seed = u16::from_le_bytes(unsafe { *(bytes.as_ptr().add(47) as *const [u8; 2]) });
        let bump = bytes[49];
        Ok(Self {
            creator,
            threshold,
            num_members,
            max_expiry_duration,
            veto_threshold,
            proposal_counter,
            seed,
            bump,
        })
    }

    pub fn to_bytes(&self) -> [u8; Self::LEN] {
        let mut bytes = [0u8; Self::LEN];
        bytes[0..32].copy_from_slice(self.creator.as_ref());
        bytes[32] = self.threshold;
        bytes[33] = self.num_members;
        bytes[34..38].copy_from_slice(&self.max_expiry_duration.to_le_bytes());
        bytes[38] = self.veto_threshold;
        bytes[39..47].copy_from_slice(&self.proposal_counter.to_le_bytes());
        bytes[47..49].copy_from_slice(&self.seed.to_le_bytes());
        bytes[49] = self.bump;
        bytes
    }
}

pub fn check_admin_action(payer: &Pubkey, multisig_data: &[u8]) -> Result<(), ProgramError> {
    let member_bytes = &multisig_data[Multisig::LEN..];
    for member in member_bytes.chunks(Member::LEN) {
        let member_data = Member::from_bytes(member)?;
        if member_data.pubkey == *payer && member_data.role == 1 {
            return Ok(());
        }
    }
    Err(MultisigError::NotAdmin.into())
}

pub fn get_admin_index_if_exists(payer: &Pubkey, multisig_data: &[u8]) -> Result<u8, ProgramError> {
    let member_bytes = &multisig_data[Multisig::LEN..];
    for member in member_bytes.chunks(Member::LEN) {
        let member_data = Member::from_bytes(member)?;
        if member_data.pubkey == *payer && member_data.role == 1 {
            return Ok(member_data.member_id);
        }
    }
    Err(MultisigError::NotAdmin.into())
}
