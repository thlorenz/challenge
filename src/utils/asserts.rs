use solana_program::{entrypoint::ProgramResult, msg, pubkey::Pubkey};

use crate::{error::ChallengeError, Solution};

pub fn assert_keys_equal(
    provided_key: &Pubkey,
    expected_key: &Pubkey,
    msg: &str,
) -> ProgramResult {
    if provided_key.ne(expected_key) {
        msg!("Err: {}", msg);
        msg!("Err: provided {} expected {}", provided_key, expected_key);
        Err(ChallengeError::ProvidedAtaIsIncorrect.into())
    } else {
        Ok(())
    }
}

pub fn assert_max_supported_solutions(solutions: &[Solution]) -> ProgramResult {
    let len = solutions.len();
    if len > u8::MAX as usize {
        msg!(
            "Err: solutions len ({}) is greater than maximum supported solutions ({})",
            len,
            u8::MAX
        );
        Err(ChallengeError::ExceedingMaxSupportedSolutions.into())
    } else {
        Ok(())
    }
}

pub fn assert_can_add_solutions(
    solutions: &[Solution],
    extra_solutions: &[Solution],
) -> ProgramResult {
    let solutions_len = solutions.len();
    let extra_solutions_len = extra_solutions.len();

    let final_len = solutions_len.saturating_add(extra_solutions_len);
    if final_len > u8::MAX as usize {
        msg!(
            "Err: adding {} solutions would result in {} total solutions which exceeds max supported {}",
            extra_solutions_len,
            final_len,
            u8::MAX
        );
        Err(ChallengeError::ExceedingMaxSupportedSolutions.into())
    } else {
        Ok(())
    }
}
