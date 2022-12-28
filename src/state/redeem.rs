use shank::ShankAccount;
use solana_program::pubkey::Pubkey;

use crate::challenge_id;

#[derive(ShankAccount)]
#[seeds("challenge", challenge_pda("The PDA of the challenge"))]
pub struct Redeem {}

impl Redeem {
    pub fn pda(challenge_pda: &Pubkey) -> (Pubkey, u8) {
        Redeem::shank_pda(&challenge_id(), challenge_pda)
    }
}
