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

pub fn assert_max_allocated_solutions(
    max_solutions: u8,
    solutions: &[Solution],
) -> ProgramResult {
    let provided_solutions_len = solutions.len();
    if max_solutions == 0 {
        msg!("Err: max_solutions need to be at least 1");
        Err(ChallengeError::InvalidMaxSolutions.into())
    } else if (max_solutions as usize) < provided_solutions_len {
        msg!(
            "Err: max_solutions is less than the number of provided solutions"
        );
        Err(ChallengeError::InvalidMaxSolutions.into())
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

pub fn assert_can_add_solutions_at_index(
    solutions: &[Solution],
    extra_solutions: &[Solution],
    index: u8,
    max_solutions: u8,
) -> ProgramResult {
    assert_max_supported_solutions(extra_solutions)?;
    let extra_solutions_len = extra_solutions.len() as u8;

    let final_index = index + extra_solutions_len;
    if final_index > max_solutions {
        msg!("Err: adding {} solutions at index {} would exceed max_solutions {}", extra_solutions_len, index, max_solutions);
        Err(ChallengeError::ExceedingMaxAllocatedSolutions.into())
    } else if index as usize > solutions.len() {
        msg!("Err: Adding solutions at index ({}) would create a hole in the solutions array of len ({})", index, solutions.len());
        Err(ChallengeError::SolutionArrayCannotBeSparse.into())
    } else {
        Ok(())
    }
}
