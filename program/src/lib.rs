use solana_program::{declare_id, hash::HASH_BYTES, pubkey::Pubkey};

mod entrypoint;
mod error;
pub mod ixs;
mod processor;
pub mod shank_utils;
pub mod state;
pub mod utils;

declare_id!("FFFFaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

pub fn challenge_id() -> Pubkey {
    id()
}

pub type Solution = [u8; HASH_BYTES];
