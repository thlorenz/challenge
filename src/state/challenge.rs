use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::{
    account_info::AccountInfo,
    hash::{hash, HASH_BYTES},
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

use crate::{
    challenge_id,
    utils::{assert_is_signer, assert_keys_equal},
    Solution,
};

use super::{
    HasPda, HasSize, StateFromPdaAccountValue, TryStateFromPdaAccount,
};

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

    /// Indicates if the challenge has started and is ready to accept challengers.
    /// If not it won't admit nor redeem to anyone.
    pub started: bool,

    /// Indicates if the challenge has finished.
    /// At this point no challengers can be admitted nor can any one redeem the price.
    pub finished: bool,

    /// The fee that will be transferred to the creator from the challenger account
    /// when the admit instruction is processed.
    pub admit_cost: u64,

    /// Determines how many solutions a challenger can send per admission to try to redeem.
    pub tries_per_admit: u8,

    // TODO(thlorenz): make sure this works for NFTS or create an NFTChallenge
    // which is the same thing except that redeem integrates with TokenMetadata program
    //
    /// The address of the price token.
    /// Should this be an array/collection for case b) of the reason to have multiple solutions?
    /// See below ([Challenge::solutions])
    pub redeem: Pubkey,

    /// The index of the solution that needs to be found next
    pub solving: u8,

    /// All solutions of the challenge, solving each will result in the redeem
    /// to be sent to the challenger.
    /// There are two reasons why multiple solutions exist:
    /// - a) to include a nonce for each and thus prevent challenger 2 just repeating the hashed
    ///   solution that challenger 1 provided
    /// - b) there is a series of puzzles that can be solved in order to solve the challenge, and
    ///   challengers may be allowed to redeem multiple times and receive the `redeem` token more
    ///   than once
    pub solutions: Vec<Solution>,
}

impl std::fmt::Debug for Challenge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Challenge")
            .field("authority", &self.authority)
            .field("id", &self.id)
            .field("started", &self.started)
            .field("finished", &self.finished)
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
    /* started */         1 +
    /* finished */        1 +
    /* admit_cost */      8 +
    /* tries_per_admit */ 1 +
    /* redeem */         32 +
    /* solving */         1 +
    /* solutions */       4; // u32 for Vec::len

impl HasSize for Challenge {
    /// Returns the size assuming no more solutions will be added.
    fn size(&self) -> usize {
        Challenge::needed_size(&self.solutions, &self.id)
    }
}

impl HasPda for Challenge {
    fn pda(&self) -> (Pubkey, u8) {
        Challenge::shank_pda(&challenge_id(), &self.authority, &self.id)
    }
}

impl Challenge {
    pub fn needed_size(solutions: &[Solution], id: &str) -> usize {
        EMPTY_CHALLENGE_SIZE_WITH_EMPTY_ID
            + id.len()
            + Challenge::space_to_store_n_solutions(solutions.len() as u8)
    }

    pub fn space_to_store_n_solutions(solutions_len: u8) -> usize {
        solutions_len as usize * HASH_BYTES
    }

    /// Only use on-chain as Rent::get is not available otherwise.
    #[allow(unused)]
    pub(crate) fn rent_exempt_lamports(&self) -> Result<u64, ProgramError> {
        let rent = Rent::get()?;
        Ok(rent.minimum_balance(self.size()))
    }

    /// Deserializes a challenge from the given account data and verifies the following:
    /// - the provided challenge pda account is for the provided creator and challenge id  
    /// - the challenge account is funded and initialized (has data)
    /// - the creator (authority) is signer
    /// - the creator is the authority for the challenge
    pub fn account_state_verifying_creator(
        challenge_pda_info: &AccountInfo,
        creator_info: &AccountInfo,
        id: &str,
    ) -> Result<StateFromPdaAccountValue<Challenge>, ProgramError> {
        let StateFromPdaAccountValue::<Challenge> { state, pda, bump } =
            challenge_pda_info.try_state_from_pda_account(|| {
                Challenge::shank_pda(&challenge_id(), creator_info.key, id)
            })?;

        assert_is_signer(creator_info, "creator")?;

        assert_keys_equal(&state.authority, creator_info.key, || {
            format!(
            "Challenge's authority ({}) does not match provided creator ({})",
            state.authority, creator_info.key
        )
        })?;
        Ok(StateFromPdaAccountValue::<Challenge> { state, pda, bump })
    }

    pub fn current_solution(&self) -> Option<&Solution> {
        self.solutions.get(self.solving as usize)
    }

    pub fn is_solution_correct(&self, sent_solution: &Solution) -> bool {
        let solution_stored_as = hash(sent_solution).to_bytes();
        let correct_solution = self.current_solution();

        // We should always get a solution here since we assert first that we have one
        if let Some(correct_solution) = correct_solution {
            solution_stored_as.eq(correct_solution)
        } else {
            false
        }
    }
}
