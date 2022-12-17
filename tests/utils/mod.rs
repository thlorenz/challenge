use challenge::challenge_id;
use solana_program_test::ProgramTest;

pub mod ixs_custom;

pub fn program_test() -> ProgramTest {
    ProgramTest::new("challenge", challenge_id(), None)
}
