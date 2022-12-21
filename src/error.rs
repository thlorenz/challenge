use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

#[derive(Clone, Debug, Eq, Error, PartialEq, FromPrimitive)]
pub enum ChallengeError {
    #[error("You are not authorized to perform this action.")]
    Unauthorized = 0x11c7ac,

    #[error("Account should be signer")]
    AccountShouldBeSigner,

    #[error("Provided PDA does not match the expected PDA")]
    ProvidedPdaIsIncorrect,

    #[error("Provided ATA does not match the expected ATA")]
    ProvidedAtaIsIncorrect,

    #[error("Account should be program")]
    AccountShouldBeProgram,

    // -----------------
    // Adding Solutions
    // -----------------
    #[error("Amount of solutions exceeds maximum supported solutions ")]
    ExceedingMaxSupportedSolutions,

    #[error("Adding solutions would exceed max_solutions")]
    ExceedingMaxAllocatedSolutions,

    #[error("Account should have address")]
    AccountShouldHaveAddress,

    #[error(
        "ATA Account is not correctly derived from the Challenger Account"
    )]
    ATAIsNotForChallenger,

    #[error("ATA Account data is empty")]
    ATAIsNotInitialized,

    #[error("Payer does not have sufficient lamports to fund the operation")]
    InsufficientFunds,
}

impl PrintProgramError for ChallengeError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl From<ChallengeError> for ProgramError {
    fn from(e: ChallengeError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for ChallengeError {
    fn type_of() -> &'static str {
        "TokenOwner Error"
    }
}
