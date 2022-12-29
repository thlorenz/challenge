use challenge::{challenge_id, Solution};
use solana_program::hash::hash;
use solana_program_test::ProgramTest;

mod accounts;
mod mints;
pub use accounts::*;
pub use mints::*;
pub mod ixs_custom;

pub fn program_test() -> ProgramTest {
    ProgramTest::new("challenge", challenge_id(), None)
}

/// Mimics the `sha256(sha256(solution))` that is performed on each solution passed
/// to ixs::create_challenge.
#[allow(unused)] // it actually is in 01_create_challenge.rs
pub fn hash_solution(solution: &str) -> Solution {
    let users_sends = hash(solution.as_bytes()).to_bytes();
    // program stores
    hash(&users_sends).to_bytes()
}
