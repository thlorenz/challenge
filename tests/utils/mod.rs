use challenge::challenge_id;
use solana_program_test::ProgramTest;

mod accounts;
pub use accounts::*;
pub mod ixs_custom;

pub fn program_test() -> ProgramTest {
    ProgramTest::new("challenge", challenge_id(), None)
}
