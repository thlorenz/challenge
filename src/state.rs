use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::{
    hash::HASH_BYTES, program_error::ProgramError, pubkey::Pubkey, rent::Rent,
    sysvar::Sysvar,
};

use crate::Solution;

#[derive(ShankAccount, BorshSerialize, BorshDeserialize)]
#[seeds(
    "challenge",
    creator("The authority managing the challenge, usually the creator"),
    challenge_id(
        "Unique id of the challenge. The same creator cannot reuse the same id for different challenges.",
        str
    )
)]
/// This is the PDA account that holds the state of a challenge.
/// The creator will usually be the update_authority, but this is not required.
///
/// The `authority` is used as the creator seed when deriving the PDA of this challenge.
///   - it is not needed for all state changes, i.e. the `solving` will be incremented via the
///     without requiring the signature of the `authority`.
///   - however adding solutions requires the authority to sign
pub struct Challenge {
    /// The authority that can update the challenge, normally the creator.
    pub authority: Pubkey,

    /// The id of the challenge, needs to be unique for the creator.
    pub id: String,

    /// Indicates if the challenge is ready to accept challengers.
    /// If not it won't admit nor redeem to anyone.
    pub ready: bool,

    /// The fee that will be transferred to the creator from the challenger account
    /// when the admit instruction is processed.
    pub admit_cost: u64,

    /// Determines how many solutions a challenger can send per admission to try to redeem.
    pub tries_per_admit: u8,

    // TODO(thlorenz): make sure this works for NFTS or create an NFTChallenge
    // which is the same thing except that redeem integrates with TokenMetadata program
    pub redeem: Pubkey,

    /// The index of the solution that needs to be found next
    pub solving: u8,

    /// All solutions of the challenge, solving each will result in the redeem
    /// to be sent to the challenger.
    pub solutions: Vec<Solution>,
}

impl std::fmt::Debug for Challenge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Challenge")
            .field("authority", &self.authority)
            .field("id", &self.id)
            .field("admit_cost", &self.admit_cost)
            .field("tries_per_admit", &self.tries_per_admit)
            .field("redeem", &self.redeem)
            .field("solving", &self.solving)
            .field("solutions", &self.solutions.len())
            .finish()
    }
}

#[rustfmt::skip]
pub const EMPTY_CHALLENGE_SIZE_WITH_EMPTY_ID: usize =
    /* authority */      32 + 
    /* id */              4 + /* does not include string len */
    /* ready */           1 +
    /* admit_cost */      8 +
    /* tries_per_admit */ 1 +
    /* redeem */         32 +
    /* solving */         1 +
    /* solutions */       4; // u32 for Vec::len

impl Challenge {
    pub fn needed_size(solutions: &[Solution], id: &str) -> usize {
        EMPTY_CHALLENGE_SIZE_WITH_EMPTY_ID
            + id.len()
            + Challenge::space_to_store_n_solutions(solutions.len() as u8)
    }

    pub fn space_to_store_n_solutions(solutions_len: u8) -> usize {
        solutions_len as usize * HASH_BYTES
    }

    /// Returns the size assuming no more solutions will be added.
    pub fn size(&self) -> usize {
        Challenge::needed_size(&self.solutions, &self.id)
    }

    /// Only use on-chain as Rent::get is not available otherwise.
    #[allow(unused)]
    pub(crate) fn rent_exempt_lamports(&self) -> Result<u64, ProgramError> {
        let rent = Rent::get()?;
        Ok(rent.minimum_balance(self.size()))
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
