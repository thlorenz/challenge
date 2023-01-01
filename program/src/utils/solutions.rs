use solana_program::hash::hash;

use crate::Solution;

pub fn hash_solution_challenger_sends(s: &str) -> [u8; 32] {
    hash(s.as_bytes()).to_bytes()
}

pub fn hash_solutions(solutions: &[&str]) -> Vec<Solution> {
    solutions
        .iter()
        .map(|s| {
            let challenger_sends = hash_solution_challenger_sends(s);
            // program stores
            hash(&challenger_sends).to_bytes()
        })
        .collect::<Vec<Solution>>()
}
