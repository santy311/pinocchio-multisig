use crate::error::MultisigError;

use super::utils::DataLen;
use pinocchio::{
    msg,
    program_error::ProgramError,
    pubkey::{self, Pubkey},
    sysvars::{clock::Clock, Sysvar},
};
use shank::ShankType;

#[repr(C, align(8))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, ShankType)]
pub enum ProposalStatus {
    Pending = 0,
    Approved = 1,
    Rejected = 2,
    Vetoed = 3,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, ShankType)]
pub struct Proposal {
    pub multisig: [u8; 32],
    pub id: u64,
    pub creator: u8,
    pub status: u8,
    pub yes_votes: u8,
    pub no_votes: u8,
    pub veto_votes: u8,
    pub expiry: u64,
    pub bump: u8,
}

impl DataLen for Proposal {
    const LEN: usize = 32 + 8 + 1 + 1 + 1 + 1 + 1 + 8 + 1;
}

impl Proposal {
    pub const SEED: &'static str = "proposal";

    pub fn new(
        multisig: [u8; 32],
        id: u64,
        creator: u8,
        expiry_duration: u32,
        bump: u8,
    ) -> Result<Self, ProgramError> {
        Ok(Self {
            multisig,
            id,
            creator,
            status: ProposalStatus::Pending as u8,
            yes_votes: 0,
            no_votes: 0,
            veto_votes: 0,
            expiry: Clock::get()?.unix_timestamp as u64 + (expiry_duration as u64 * 1000),
            bump,
        })
    }

    pub fn validate_pda(
        bump: u8,
        pda: &Pubkey,
        id_seed: u64,
        owner: &Pubkey,
    ) -> Result<(), ProgramError> {
        msg!("Validating PDA");
        msg!("owner: {:?}", owner);
        msg!("id_seed: {:?}", id_seed.to_le_bytes().as_ref());
        msg!("bump: {:?}", bump);
        msg!("pda: {:?}", pda);
        let id_seed_bytes = &id_seed.to_le_bytes();
        let seed_with_bump = &[
            Self::SEED.as_bytes(),
            owner,
            id_seed_bytes.as_ref(),
            &[bump],
        ];
        let derived = pubkey::create_program_address(seed_with_bump, &crate::ID)?;
        if derived != *pda {
            msg!("derived: {:?}", derived);
            msg!("pda: {:?}", pda);
            return Err(MultisigError::PdaMismatch.into());
        }
        Ok(())
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ProgramError> {
        assert!(
            bytes.len() >= Self::LEN,
            "Not enough bytes to deserialize Proposal"
        );
        let multisig = bytes[0..32].try_into().unwrap();
        let id = u64::from_le_bytes(bytes[32..40].try_into().unwrap());
        let creator = bytes[40];
        let status = bytes[41];
        let yes_votes = bytes[42];
        let no_votes = bytes[43];
        let veto_votes = bytes[44];
        let expiry = u64::from_le_bytes(bytes[45..53].try_into().unwrap());
        let bump = bytes[53];
        Ok(Self {
            multisig,
            id,
            creator,
            status,
            yes_votes,
            no_votes,
            veto_votes,
            expiry,
            bump,
        })
    }

    pub fn to_bytes(&self) -> [u8; Self::LEN] {
        let mut bytes = [0u8; Self::LEN];
        bytes[0..32].copy_from_slice(&self.multisig);
        bytes[32..40].copy_from_slice(&self.id.to_le_bytes());
        bytes[40] = self.creator;
        bytes[41] = self.status;
        bytes[42] = self.yes_votes;
        bytes[43] = self.no_votes;
        bytes[44] = self.veto_votes;
        bytes[45..53].copy_from_slice(&self.expiry.to_le_bytes());
        bytes[53] = self.bump;
        bytes
    }

    pub fn get_status(&self) -> Result<ProposalStatus, MultisigError> {
        match self.status {
            0 => Ok(ProposalStatus::Pending),
            1 => Ok(ProposalStatus::Approved),
            2 => Ok(ProposalStatus::Rejected),
            3 => Ok(ProposalStatus::Vetoed),
            _ => Err(MultisigError::InvalidProposalStatus),
        }
    }

    pub fn vote(&mut self, vote: u8) -> Result<(), ProgramError> {
        if vote != 0 && vote != 1 {
            return Err(MultisigError::InvalidVotingOption.into());
        }
        if vote == 0 {
            self.no_votes += 1;
        } else {
            self.yes_votes += 1;
        }
        Ok(())
    }
}
