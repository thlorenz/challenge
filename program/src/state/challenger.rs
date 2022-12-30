use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::pubkey::Pubkey;

use crate::challenge_id;

use super::{HasPda, HasSize};

#[derive(Debug, ShankAccount, BorshDeserialize, BorshSerialize)]
#[seeds(
    "challenge",
    challenge_pda("The challenge PDA that the challenger wants to solve."),
    challenger("The address attempting to solve the challenge")
)]
pub struct Challenger {
    /// The authority that can redeem the challenge, which is the account from
    /// which the challenger PDA (the owner of this account) was derived.
    pub authority: Pubkey,

    /// The PDA of the challenge that the challenger is solving.
    pub challenge_pda: Pubkey,

    /// How many more attempts the callenger has to provide a solution to redeem.
    pub tries_remaining: u8,

    /// This means that the challenger redeemed at least once.
    /// Shoud this be a count even though we could just mint multiple `redeem` tokens?
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

impl HasPda for Challenger {
    fn pda(&self) -> (Pubkey, u8) {
        Challenger::shank_pda(
            &challenge_id(),
            &self.challenge_pda,
            &self.authority,
        )
    }
}

impl Challenger {
    pub fn size() -> usize {
        CHALLENGER_SIZE
    }
}
