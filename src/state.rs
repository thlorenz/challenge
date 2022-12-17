use solana_program::{hash::HASH_BYTES, pubkey::Pubkey};

pub struct Challenge {
    pub admit_cost: u64,
    pub tries_per_admit: u64,

    pub redeem: Pubkey,

    pub solving: u8,
    pub solutions: Vec<[u8; HASH_BYTES]>,
}

pub struct Challenger {
    pub tries_remaining: u64,
}
