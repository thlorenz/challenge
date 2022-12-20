use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::{hash::HASH_BYTES, pubkey::Pubkey};

use crate::Solution;

#[derive(ShankAccount, BorshSerialize, BorshDeserialize)]
#[seeds(
    "challenge",
    creator("The authority managing the challenge, usually the creator")
)]
/// This is the PDA account that holds the state of a challenge.
/// The creator will usually be the update_authority, but this is not required.
///
/// The `authority` is used as the creator seed when deriving the PDA of this challenge.
///   - it is not needed for all state changes, i.e. the `solving` will be incremented via the
///     without requiring the signature of the `authority`.
///   - however adding solutions requires the authority to sign
pub struct Challenge {
    pub authority: Pubkey,

    pub admit_cost: u64,
    pub tries_per_admit: u8,

    // TODO(thlorenz): make sure this works for NFTS or create an NFTChallenge
    // which is the same thing except that redeem integrates with TokenMetadata program
    pub redeem: Pubkey,

    pub solving: u8,

    pub solutions: Vec<Solution>,
}

impl std::fmt::Debug for Challenge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Challenge")
            .field("authority", &self.authority)
            .field("admit_cost", &self.admit_cost)
            .field("tries_per_admit", &self.tries_per_admit)
            .field("redeem", &self.redeem)
            .field("solving", &self.solving)
            .field("solutions", &self.solutions.len())
            .finish()
    }
}

#[rustfmt::skip]
pub const EMPTY_CHALLENGE_SIZE: usize =
    /* authority */      32 + 
    /* admit_cost */      8 +
    /* tries_per_admit */ 1 +
    /* redeem */         32 +
    /* solving */         1 +
    /* solutions */       4; // u32 for Vec::len

impl Challenge {
    pub fn needed_size(solutions: &[Solution]) -> usize {
        EMPTY_CHALLENGE_SIZE
            + Challenge::space_to_store_n_solutions(solutions.len() as u8)
    }

    pub fn space_to_store_n_solutions(solutions_len: u8) -> usize {
        solutions_len as usize * HASH_BYTES
    }

    pub fn max_solutions(data_len: usize) -> u8 {
        // Alternatively we could just read the vec len at solutions index
        let els_size = data_len - EMPTY_CHALLENGE_SIZE;
        (els_size / HASH_BYTES) as u8
    }

    /// Returns the size assuming no more solutions will be added.
    pub fn size(&self) -> usize {
        Challenge::needed_size(&self.solutions)
    }
}

#[derive(Debug, ShankAccount)]
#[seeds(
    "challenge",
    challenge(
        "The challenge that the challenger is solving, i.e. the Challenge PDA"
    ),
    challenger("The address attempting to solve the challenge")
)]
pub struct Challenger {
    pub tries_remaining: u64,
}
