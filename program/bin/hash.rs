use std::env;

use challenge::utils::hash_solution_challenger_sends;

fn main() {
    let args: Vec<String> = env::args().collect();
    let s = args[1].as_str();
    let hashed = hash_solution_challenger_sends(s);
    println!("{} -> sends {:x?}", s, hashed);
}
