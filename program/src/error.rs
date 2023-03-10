use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

#[derive(Clone, Debug, Eq, Error, PartialEq, FromPrimitive)]
pub enum ChallengeError {
    // -----------------
    // Security
    // -----------------
    #[error("Account should be signer")]
    AccountShouldBeSigner = 0x11c7ac,

    #[error("Provided ATA does not match the expected ATA")]
    ProvidedAtaIsIncorrect,

    // -----------------
    // Create Challenge
    // -----------------
    #[error("Account not funded")]
    AccountNotFunded,

    // -----------------
    // Adding Solutions
    // -----------------
    #[error("Amount of solutions exceeds maximum supported solutions ")]
    ExceedingMaxSupportedSolutions,

    #[error("When adding solutions you need to provide at least one solution")]
    NoSolutionsToAddProvided,

    #[error("Account was expected to not exists yet, but it does")]
    AccountAlreadyExists,

    #[error("Account has data but was expected to be empty")]
    AccountAlreadyHasData,

    #[error("Account has no data")]
    AccountHasNoData,

    // -----------------
    // Starting Challenge
    // -----------------
    #[error("Challenge was started already and cannot be started again")]
    ChallengeAlreadyStarted,

    #[error("Challenge has no solutions and thus cannot be started")]
    ChallengeHasNoSolutions,

    // -----------------
    // Admit
    // -----------------
    #[error(
        "Challenge has not started yet and is not ready to admit challengers"
    )]
    ChallengeNotYetStarted,

    #[error("Challenge was finished already and is not admitting challengers nor allowing to redeem prices")]
    ChallengeAlreadyFinished,

    // -----------------
    // Redeem
    // -----------------
    #[error(
        "The provided solution did not match the currently expected solution"
    )]
    SolutionIsIncorrect,

    // We should never encounter this since the challenge would be auto-finished
    // after last solution was provided
    #[error("All solutions were already redeemed")]
    OutOfSolutions,

    #[error("This challenger used up all tries to solve the challenge")]
    ChallengerHasNoTriesRemaining,

    // -----------------
    // Misc
    // -----------------
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
