use shank::ShankAccount;
use solana_program::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address_with_program_id;

use crate::challenge_id;

use super::{Challenge, HasPda};

#[derive(ShankAccount)]
#[seeds("challenge", challenge_pda("The PDA of the challenge"))]
/// This account is only constructed to access convenience methods.
/// It's data is never stored on chain.
pub struct Redeem {
    /// The PDA of the challenge that uses this redeem.
    /// This address is also set to be the authority of the mint.
    pub challenge_pda: Pubkey,

    /// Derived PDA of this redeem token
    /// (the challenge_pda is the address from which it is derived).
    /// This is the actual mint address we use.
    pub pda: Pubkey,
}

impl Redeem {
    pub fn new(challenge_pda: Pubkey) -> Self {
        let (pda, _) = Redeem::shank_pda(&challenge_id(), &challenge_pda);
        Self { challenge_pda, pda }
    }

    pub fn for_challenge_with(creator: &Pubkey, id: &str) -> Self {
        let (challenge_pda, _) = Challenge::pda_for(creator, id);
        Redeem::new(challenge_pda)
    }

    pub fn ata(&self, recvr: &Pubkey) -> Pubkey {
        let ata = get_associated_token_address_with_program_id(
            recvr,
            &self.pda,
            &spl_token::id(),
        );
        eprintln!(
            "recvr: {} mint: {} ata: {}\n{}
            ",
            recvr,
            self.pda,
            ata,
            &spl_token::id(),
        );
        ata
    }
}

impl HasPda for Redeem {
    fn pda(&self) -> (Pubkey, u8) {
        Redeem::shank_pda(&challenge_id(), &self.challenge_pda)
    }
}
