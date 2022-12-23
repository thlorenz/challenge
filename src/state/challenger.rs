use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::pubkey::Pubkey;

use super::HasSize;

#[derive(Debug, ShankAccount, BorshDeserialize, BorshSerialize)]
#[seeds(
    "challenge",
    challenge_pda("The challenge PDA that the challenger wants to solve."),
    challenger("The address attempting to solve the challenge")
)]
pub struct Challenger {
    pub authority: Pubkey,
    pub challenge_pda: Pubkey,
    pub tries_remaining: u8,
    pub redeemed: bool,
}

#[rustfmt::skip]
pub const CHALLENGER_SIZE: usize =
    /* authority */      32 + 
    /* challenge_pda */  32 + 
    /* tries_remaining */ 1 +
    /* redeemed */        1;

impl HasSize for Challenger {
    fn size(&self) -> usize {
        CHALLENGER_SIZE
    }
}

impl Challenger {
    pub fn size() -> usize {
        CHALLENGER_SIZE
    }
}

/* // TODO(thlorenz): @@@ Traits first
impl Challenger {
    /// Deserializes a challenger from the given account data and verifies the following:
    /// - the provided challenger pda account is for the provided challenger and challenge PDA
    /// - the challenge account is funded and initialized (has data)
    /// - the challenger (authority) is signer
    /// - the challenger is the authority for the challenge
    pub fn mutable_from_data_verifying_challenger(
        challenge_pda_info: &AccountInfo,
        challenger_pda_info: &AccountInfo,
    ) -> Result<PdaAccountInfo<Challenger>, ProgramError> {
        let PdaAccountInfo<Challenger> { challenge, pda } =
            PdaAccountInfo::<Challenger>::mutable_from_data(challenger_pda_info)?;

        assert_keys_equal(challenger_pda_info.key, &pda, || {
            format!(
            "PDA for the challenge for creator ({}) and id ({}) is incorrect",
            creator_info.key, id
        )
        })?;
        assert_account_is_funded_and_has_data(challenger_pda_info)?;

        let challenge = {
            let challenge_data = &challenger_pda_info.try_borrow_data()?;
            try_from_slice_unchecked::<Challenge>(challenge_data)?
        };

        assert_is_signer(creator_info, "creator")?;

        assert_keys_equal(&challenge.authority, creator_info.key, || {
            format!(
            "Challenge's authority ({}) does not match provided creator ({})",
            challenge.authority, creator_info.key
        )
        })?;
        Ok(MutableChallengeFromData { challenge, pda })
    }
}
*/

/*
    creator("The authority managing the challenge. Matches Challenge PDA's creator."),
    challenge_id(
        "Unique id of the challenge. Matches Challenge PDA's id.",
        str
    ),
*/
