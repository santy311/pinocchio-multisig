use pinocchio::{
    account_info::AccountInfo, entrypoint, msg, program_error::ProgramError, pubkey::Pubkey,
    ProgramResult,
};

use crate::instruction::{
    add_member, create_proposal, init_multisig, modify_config, remove_member,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    msg!("Processing instruction");
    let (discriminator, data) = data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match discriminator {
        0 => init_multisig::process_init_multisig_instruction(accounts, data)?,
        1 => add_member::process_add_member_instruction(accounts, data)?,
        2 => modify_config::process_modify_config_instruction(accounts, data)?,
        3 => remove_member::process_remove_member_instruction(accounts, data)?,
        4 => create_proposal::process_create_proposal_instruction(accounts, data)?,
        _ => return Err(ProgramError::InvalidInstructionData),
    }
    Ok(())
}
