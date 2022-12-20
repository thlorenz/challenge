use challenge::challenge_id;
use solana_program::hash::{hash, HASH_BYTES};
use solana_program_test::ProgramTest;

mod accounts;
pub use accounts::*;
pub mod ixs_custom;

pub fn program_test() -> ProgramTest {
    ProgramTest::new("challenge", challenge_id(), None)
}

/// Mimics the `sha256(sha256(solution))` that is performed on each solution passed
/// to ixs::create_challenge.
pub fn hash_solution(solution: &str) -> [u8; HASH_BYTES] {
    let users_sends = hash(solution.as_bytes()).to_bytes();
    // program stores
    hash(&users_sends).to_bytes()
}
