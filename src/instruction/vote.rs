use pinocchio::{
    account_info::AccountInfo,
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

    let ix_data = VoteData::from_bytes(data)?;

    Multisig::validate_pda(
        ix_data.multisig_bump,
        multisig_acc.key(),
        ix_data.multisig_id,
    )?;

    Proposal::validate_pda(
        ix_data.proposal_bump,
        proposal_acc.key(),
        ix_data.proposal_id,
        multisig_acc.key(),
    )?;

    let member_index = get_member_index_if_exists(payer_acc.key(), unsafe {
        multisig_acc.borrow_data_unchecked()
    })?;

    {
        let (proposal_meta_data, voter_list_data) =
            unsafe { proposal_acc.borrow_mut_data_unchecked() }.split_at(Proposal::LEN);
        let proposal = Proposal::from_bytes(proposal_meta_data)?;
        if proposal.get_status()? != ProposalStatus::Pending {
            return Err(ProgramError::InvalidInstructionData);
        }

        for voter in voter_list_data.chunks(1) {
            let voter_id = voter[0];
            if voter_id == member_index {
                return Err(MultisigError::MemberAlreadyVoted.into());
            }
        }
    }

    let old_size = proposal_acc.data_len();
    let new_size = old_size + 1;

    let new_rent = Rent::get()?.minimum_balance(new_size);
    let rent_diff = new_rent - proposal_acc.lamports();
    if rent_diff > 0 {
        Transfer {
            from: &payer_acc,
            to: &proposal_acc,
            lamports: rent_diff,
        }
        .invoke()?;
    }

    proposal_acc.resize(new_size)?;

    let (proposal_data, voter_list_data) =
        unsafe { proposal_acc.borrow_mut_data_unchecked() }.split_at_mut(Proposal::LEN);
    let mut proposal = Proposal::from_bytes(proposal_data)?;

    let yes_votes = proposal.yes_votes as usize;
    let no_votes = proposal.no_votes as usize;
    let veto_votes = proposal.veto_votes as usize;
    let total_votes = yes_votes + no_votes + veto_votes;

    match ix_data.vote {
        0 => {
            // No vote: insert at end of no section, shift veto right
            voter_list_data
                .copy_within(yes_votes + no_votes..total_votes, yes_votes + no_votes + 1);
            voter_list_data[yes_votes + no_votes] = member_index;
            proposal.no_votes += 1;
        }
        1 => {
            // Yes vote: insert at end of yes section, shift no+veto right
            voter_list_data.copy_within(yes_votes..total_votes, yes_votes + 1);
            voter_list_data[yes_votes] = member_index;
            proposal.yes_votes += 1;
        }
        2 => {
            // Veto vote: append at end
            voter_list_data[total_votes] = member_index;
            proposal.veto_votes += 1;
        }
        _ => return Err(ProgramError::InvalidInstructionData),
    }

    unsafe {
        proposal_acc.borrow_mut_data_unchecked()[..Proposal::LEN]
            .copy_from_slice(&proposal.to_bytes());
    }

    Ok(())
}

pub struct VoteData {
    pub proposal_id: u64,
    pub vote: u8,
    pub multisig_bump: u8,
    pub proposal_bump: u8,
    pub multisig_id: u64,
}

impl DataLen for VoteData {
    const LEN: usize = 8 + 1 + 1 + 1 + 8;
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
            multisig_id: u64::from_le_bytes([
                bytes[11], bytes[12], bytes[13], bytes[14], bytes[15], bytes[16], bytes[17],
                bytes[18],
            ]),
        })
    }

    pub fn to_bytes(&self) -> [u8; Self::LEN] {
        let mut bytes = [0u8; Self::LEN];
        bytes[0..8].copy_from_slice(&self.proposal_id.to_le_bytes());
        bytes[8] = self.vote;
        bytes[9] = self.multisig_bump;
        bytes[10] = self.proposal_bump;
        bytes[11..19].copy_from_slice(&self.multisig_id.to_le_bytes());
        bytes
    }
}
