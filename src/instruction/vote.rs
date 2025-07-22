use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    msg,
    program_error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::Transfer;

use crate::{
    error::MultisigError,
    state::{get_member_index_if_exists, DataLen, Multisig, Proposal, ProposalStatus},
};

pub fn process_vote_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [payer_acc, multisig_acc, proposal_acc, _remaining_accounts @ ..] = accounts else {
        return Err(ProgramError::InvalidInstructionData);
    };

    if !payer_acc.is_signer() {
        return Err(ProgramError::InvalidInstructionData);
    }

    msg!("Processing vote instruction");
    let ix_data = VoteData::from_bytes(data)?;

    msg!("Validating PDA");
    Multisig::validate_pda(ix_data.multisig_bump, multisig_acc.key(), payer_acc.key())?;
    msg!("Validating proposal PDA");
    Proposal::validate_pda(
        ix_data.proposal_bump,
        proposal_acc.key(),
        ix_data.proposal_id,
        multisig_acc.key(),
    )?;

    msg!("Getting member index");
    let member_index = get_member_index_if_exists(payer_acc.key(), unsafe {
        multisig_acc.borrow_data_unchecked()
    })?;

    {
        msg!("Checking proposal status");
        let (proposal_meta_data, mut voter_list_data) =
            unsafe { proposal_acc.borrow_mut_data_unchecked() }.split_at(Proposal::LEN);
        let mut proposal = Proposal::from_bytes(proposal_meta_data)?;
        if proposal.get_status()? != ProposalStatus::Pending {
            return Err(ProgramError::InvalidInstructionData);
        }

        msg!("Checking if member already voted");
        for voter in voter_list_data.chunks(1) {
            let voter_id = voter[0];
            if voter_id == member_index {
                return Err(MultisigError::MemberAlreadyVoted.into());
            }
        }
    }

    msg!("Adding member to voter list");
    let old_size = proposal_acc.data_len();
    let new_size = old_size + 1;

    let new_rent = Rent::get()?.minimum_balance(new_size);
    let rent_diff = new_rent - proposal_acc.lamports();
    if rent_diff > 0 {
        msg!("Transferring rent");
        Transfer {
            from: &payer_acc,
            to: &proposal_acc,
            lamports: rent_diff,
        }
        .invoke()?;
    }

    msg!("Resizing proposal account");
    msg!("old_size: {:?}", old_size);
    msg!("new_size: {:?}", new_size);
    proposal_acc.resize(new_size)?;

    msg!("new accounts: {:?}", proposal_acc.data_len());
    let (proposal_data, voter_list_data) =
        unsafe { proposal_acc.borrow_mut_data_unchecked() }.split_at_mut(Proposal::LEN);
    let mut proposal = Proposal::from_bytes(proposal_data)?;

    msg!("Voting");
    let old_size = proposal_acc.data_len() - Proposal::LEN - 1;
    if ix_data.vote == 0 {
        voter_list_data[old_size] = member_index;
    } else {
        msg!("Copying voter list");
        voter_list_data.copy_within(
            (proposal.yes_votes as usize)..,
            (proposal.yes_votes as usize) + 1,
        );
        voter_list_data[old_size] = member_index;
    }
    proposal.vote(ix_data.vote)?;

    Ok(())
}

pub struct VoteData {
    pub proposal_id: u64,
    pub vote: u8,
    pub multisig_bump: u8,
    pub proposal_bump: u8,
}

impl DataLen for VoteData {
    const LEN: usize = 8 + 1 + 1 + 1;
}

impl VoteData {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ProgramError> {
        assert!(
            bytes.len() >= Self::LEN,
            "Not enough bytes to deserialize VoteData"
        );
        Ok(Self {
            proposal_id: u64::from_le_bytes(bytes[0..8].try_into().unwrap()),
            vote: bytes[8],
            multisig_bump: bytes[9],
            proposal_bump: bytes[10],
        })
    }

    pub fn to_bytes(&self) -> [u8; Self::LEN] {
        let mut bytes = [0u8; Self::LEN];
        bytes[0..8].copy_from_slice(&self.proposal_id.to_le_bytes());
        bytes[8] = self.vote;
        bytes[9] = self.multisig_bump;
        bytes[10] = self.proposal_bump;
        bytes
    }
}
