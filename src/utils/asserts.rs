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
