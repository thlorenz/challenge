use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::{hash::HASH_BYTES, pubkey::Pubkey};

#[derive(Debug, ShankAccount, BorshSerialize, BorshDeserialize)]
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

    pub solutions: Vec<[u8; HASH_BYTES]>,
}

pub const EMPTY_CHALLENGE_SIZE: usize = 
    /* authority */      32 + 
    /* admit_cost */      8 +
    /* tries_per_admit */ 1 +
    /* redeem */         32 +
    /* solving */         1 +
    /* solutions */       0;

impl Challenge {
    pub fn size(max_solutions: u8) -> usize {
        EMPTY_CHALLENGE_SIZE + max_solutions as usize * HASH_BYTES
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
