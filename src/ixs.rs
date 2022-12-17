use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    hash::HASH_BYTES,
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{challenge_id, state::Challenge};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum ChallengeInstruction {
    CreateChallenge {
        admit_cost: u64,
        tries_per_admit: u64,
        redeem: Pubkey,
        solutions: Option<Vec<[u8; HASH_BYTES]>>,
    },

    AddSolutions {
        solutions: Vec<[u8; HASH_BYTES]>,
    },
}

// -----------------
// Create Challenge
// -----------------
/// Creates a new challenge and is invoked only once by the creator of the challenge.
///
/// [payer]: pays for the transaction and is usually the creator
/// [creator]: the authority managing the challenge, usually the creator
/// [admit_cost]: the amount of SOL that must be paid to admit a challenger
/// [tries_per_admit]: the number of tries that a challenger gets for the given admit_cost
/// [redeem]: the address that will receive the SOL when a solution of the challenge found
/// [solutions]: the solutions need to be solved, encoded as follows `sha256(sha256(solution))`
pub fn create_challenge(
    payer: Pubkey,
    creator: Pubkey,
    admit_cost: u64,
    tries_per_admit: u64,
    redeem: Pubkey,
    solutions: Option<Vec<[u8; HASH_BYTES]>>,
) -> Result<Instruction, ProgramError> {
    let (challenge_pda, _) = Challenge::shank_pda(&challenge_id(), &creator);
    let ix = Instruction {
        program_id: challenge_id(),
        accounts: vec![
            AccountMeta::new_readonly(creator, false),
            AccountMeta::new(challenge_pda, false),
        ],
        data: ChallengeInstruction::CreateChallenge {
            admit_cost,
            tries_per_admit,
            redeem,
            solutions,
        }
        .try_to_vec()?,
    };

    Ok(ix)
}
