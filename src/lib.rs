use solana_program::{declare_id, pubkey::Pubkey};

mod entrypoint;
mod error;
pub mod ixs;
mod processor;
pub mod state;
mod utils;

declare_id!("FFFFaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

pub fn challenge_id() -> Pubkey {
    id()
}
