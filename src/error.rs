use pinocchio::program_error::ProgramError;

#[derive(Clone, PartialEq, shank::ShankType)]
pub enum MultisigError {
    // overflow error
    WriteOverflow,
    // invalid instruction data
    InvalidInstructionData,
    // pda mismatch
    PdaMismatch,
    // Invalid Owner
    InvalidOwner,
    // Multisig already initialized
    MultisigAlreadyInitialized,
    // Member not found
    MemberNotFound,
}

impl From<MultisigError> for ProgramError {
    fn from(e: MultisigError) -> Self {
        Self::Custom(e as u32)
    }
}
