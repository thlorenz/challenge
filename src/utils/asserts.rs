use solana_program::{entrypoint::ProgramResult, msg, pubkey::Pubkey};

use crate::error::ChallengeError;

pub fn assert_pda(
    provided_pda: &Pubkey,
    expected_pda: &Pubkey,
    msg: &str,
) -> ProgramResult {
    if provided_pda.ne(expected_pda) {
        msg!("Err: {}", msg);
        msg!("Err: provided {} expected {}", provided_pda, expected_pda);
        Err(ChallengeError::ProvidedAtaIsIncorrect.into())
    } else {
        Ok(())
    }
}

pub fn assert_max_solutions(max_solutions: u8) -> ProgramResult {
    if max_solutions == 0 {
        msg!("Err: max_solutions need to be at least 1");
        Err(ChallengeError::InvalidMaxSolutions.into())
    } else {
        Ok(())
    }
}
