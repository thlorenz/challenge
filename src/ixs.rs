use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program,
};

use crate::{challenge_id, state::Challenge, utils::hash_solutions, Solution};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum ChallengeInstruction {
    CreateChallenge {
        admit_cost: u64,
        tries_per_admit: u8,
        redeem: Pubkey,

        /// Each solution is a hash array of of 32 bytes.
        /// Thus the max size of solutions is 32 * 256 = 8,192 bytes.
        /// Transaction size is ~1,024 bytes which means if more solutions are desired they
        /// need to be added separately via the `AddSolutions` instruction.
        solutions: Vec<Solution>,

        /// The number of solutions to expect (we allow up to 256)
        /// If this is not provided than the count of the provided is assumed to be the
        /// max amount and thus no solutions can be added in the future.
        // TODO(thlorenz): we should use AccountInfo::realloc to allow for arbitrary
        //   addition of solutions without specifying exactly how much we're planning
        //   to add
        max_solutions: Option<u8>,
    },

    AddSolutions {
        /// The solutions to add to the challenge
        solutions: Vec<Solution>,
        /// The index at which to insert them
        /// If this is not provided then solutions will be appended to existing solutions.
        /// Including it allows replacing existing solutions.
        index: Option<u8>,
    },
}

// -----------------
// Create Challenge
// -----------------

/// Creates a new challenge and is invoked only once by the creator of the challenge.
///
/// * [payer]: pays for the transaction and is usually the creator
/// * [creator]: the authority managing the challenge
/// * [admit_cost]: the amount of SOL that must be paid to admit a challenger
/// * [tries_per_admit]: the number of tries that a challenger gets for the given admit_cost
/// * [redeem]: the address that will receive the SOL when a solution of the challenge found
/// * [solutions]: solutions to be solved in clear text, they are encoded via
///   `sha256(sha256(solution))` before being passed on to the program
/// * [max_solutions]: the number of solutions to expect (we allow up to 256)
///   needs to be provided if not all solutions are provided, but will be added later
pub fn create_challenge(
    payer: Pubkey,
    creator: Pubkey,
    admit_cost: u64,
    tries_per_admit: u8,
    redeem: Pubkey,
    solutions: Vec<&str>,
    max_solutions: Option<u8>,
) -> Result<Instruction, ProgramError> {
    let (challenge_pda, _) = Challenge::shank_pda(&challenge_id(), &creator);
    let solutions = hash_solutions(&solutions);

    let ix = Instruction {
        program_id: challenge_id(),
        accounts: vec![
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(creator, false),
            AccountMeta::new(challenge_pda, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: ChallengeInstruction::CreateChallenge {
            admit_cost,
            tries_per_admit,
            redeem,
            solutions,
            max_solutions,
        }
        .try_to_vec()?,
    };

    Ok(ix)
}

// -----------------
// Add Solutions
// -----------------

/// Adds solutions to an existing challenge and can be invoked multiple times.
///
/// * [creator]: the authority managing the challenge
/// * [solutions]: solutions to be added in clear text, they are encoded via
///   `sha256(sha256(solution))` before being passed on to the program
/// * [index]: the index at which to insert the solutions
///   if provided solutions starting at that index are replaced, otherwise they are appended
pub fn add_solutions(
    creator: Pubkey,
    solutions: Vec<&str>,
    index: Option<u8>,
) -> Result<Instruction, ProgramError> {
    let (challenge_pda, _) = Challenge::shank_pda(&challenge_id(), &creator);
    let solutions = hash_solutions(&solutions);

    let ix = Instruction {
        program_id: challenge_id(),
        accounts: vec![
            AccountMeta::new_readonly(creator, true),
            AccountMeta::new(challenge_pda, false),
            AccountMeta::new_readonly(challenge_id(), false),
        ],
        data: ChallengeInstruction::AddSolutions { solutions, index }
            .try_to_vec()?,
    };

    Ok(ix)
}
