use solana_program::hash::hash;

use crate::Solution;

pub fn hash_solutions(solutions: &[&str]) -> Vec<Solution> {
    solutions
        .iter()
        .map(|s| {
            let users_sends = hash(s.as_bytes()).to_bytes();
            // program stores
            hash(&users_sends).to_bytes()
        })
        .collect::<Vec<Solution>>()
}
