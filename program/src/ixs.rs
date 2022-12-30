use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankInstruction;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program,
};

use crate::{
    challenge_id,
    state::{Challenge, Challenger, HasPda, Redeem},
    utils::{hash_solution_challenger_sends, hash_solutions},
};

#[derive(BorshSerialize, BorshDeserialize, Debug, ShankInstruction)]
pub enum ChallengeInstruction {
    #[rustfmt::skip]
    #[account(0, name = "payer", mut, sig, desc="pays for the transaction")]
    #[account(1, name = "creator", desc="challenge authority")]
    #[account(2, name = "challenge_pda", mut, desc="PDA for the challenge")]
    #[account(3, name = "redeem_pda", mut, desc="PDA of token to redeem for correct solution")]
    #[account(4, name = "token_program", desc="Token Program")]
    #[account(5, name = "system_program", desc="System Program")]
    CreateChallenge {
        id: String,
        admit_cost: u64,
        tries_per_admit: u8,

        /// The PDA address of the mint that each challenger that solves the challenge receives.
        /// It is derived from the challenge PDA.
        redeem: Pubkey,

        /// Each solution is a hash array of of 32 bytes.
        /// Thus the max size of solutions is 32 * 256 = 8,192 bytes.
        /// Transaction size is ~1,024 bytes which means if more solutions are desired they
        /// need to be added separately via the `AddSolutions` instruction.
        solutions: Vec<[u8; 32]>,
    },

    /// Appends solutions to the end of the solutions array, keeping existing solutions in place.
    #[rustfmt::skip]
    #[account(0, name = "payer", mut, sig, desc="pays for the transaction")]
    #[account(1, name = "creator", sig, desc="challenge authority")]
    #[account(2, name = "challenge_pda", mut, desc="PDA for the challenge")]
    #[account(3, name = "system_program", desc="System Program")]
    AddSolutions {
        id: String,
        /// The solutions to add to the challenge
        solutions: Vec<[u8; 32]>,
    },

    #[rustfmt::skip]
    #[account(0, name = "creator", sig, desc="challenge authority")]
    #[account(1, name = "challenge_pda", mut, desc="PDA for the challenge")]
    StartChallenge {
        id: String,
    },

    #[rustfmt::skip]
    #[account(0, name = "payer", mut, sig, desc="pays for the transaction")]
    #[account(1, name = "creator", mut, desc="challenge authority")]
    #[account(2, name = "challenge_pda", desc="PDA for the challenge")]
    #[account(3, name = "challenger", desc="challenger account which receives the redeemed token")]
    #[account(4, name = "challenger_pda", mut, desc="PDA for the challenger")]
    #[account(5, name = "system_program", desc="System Program")]
    AdmitChallenger {
        challenge_pda: Pubkey,
    },

    #[rustfmt::skip]
    #[account(0, name = "payer", mut, sig, desc="pays for the transaction")]
    #[account(1, name = "challenge_pda", mut, desc="PDA for the challenge")]
    #[account(2, name = "challenger", sig, desc="challenger account which receives the redeemed token")]
    #[account(3, name = "challenger_pda", mut, desc="PDA for the challenger")]
    #[account(4, name = "redeem", mut, desc="PDA of token to redeem for correct solution")]
    #[account(5, name = "redeem_ata", mut, desc="ATA for redeem PDA and challenger")]
    #[account(6, name = "token_program", desc="Token Program")]
    #[account(7, name = "associated_token_program", desc="Associated Token Program")]
    #[account(8, name = "system_program", desc="System Program")]
    Redeem {
        solution: [u8; 32],
    },
    // TODO(thlorenz): may need some ixs for creators that want to mutate solutions, i.e.
    //  - add solutions at index (replacing existing ones)
    //  - replace solution at index
    //  - clear solutions
}

// -----------------
// Create Challenge
// -----------------

/// Creates a new challenge and is invoked only once by the creator of the challenge.
///
/// * [payer]: pays for the transaction and is usually the creator
/// * [creator]: the authority managing the challenge
/// * [id]: unique id identifying the challenge. The same creator cannot reuse ids for different challenges
/// * [admit_cost]: the amount of SOL that must be paid to admit a challenger
/// * [tries_per_admit]: the number of tries that a challenger gets for the given admit_cost
/// * [solutions]: solutions to be solved in clear text, they are encoded via
///   `sha256(sha256(solution))` before being passed on to the program
pub fn create_challenge(
    payer: Pubkey,
    creator: Pubkey,
    id: String,
    admit_cost: u64,
    tries_per_admit: u8,
    solutions: Vec<&str>,
) -> Result<Instruction, ProgramError> {
    let (challenge_pda, _) =
        Challenge::shank_pda(&challenge_id(), &creator, &id);

    let redeem = Redeem::new(challenge_pda);
    let (redeem_pda, _) = redeem.pda();

    let solutions = hash_solutions(&solutions);

    let ix = Instruction {
        program_id: challenge_id(),
        accounts: vec![
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(creator, false),
            AccountMeta::new(challenge_pda, false),
            AccountMeta::new(redeem_pda, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: ChallengeInstruction::CreateChallenge {
            id,
            admit_cost,
            tries_per_admit,
            redeem: redeem_pda,
            solutions,
        }
        .try_to_vec()?,
    };

    Ok(ix)
}

// -----------------
// Add Solutions
// -----------------

/// Adds solutions to an existing challenge and can be invoked multiple times.
///
/// * [payer]: pays for the transaction and is usually the creator
/// * [creator]: the authority managing the challenge
/// * [id]: unique id used when creating the challenge
/// * [solutions]: solutions to be added in clear text, they are encoded via
///   `sha256(sha256(solution))` before being stored
/// * [index]: the index at which to insert the solutions
///   if provided solutions starting at that index are replaced, otherwise they are appended
pub fn add_solutions(
    payer: Pubkey,
    creator: Pubkey,
    id: String,
    solutions: Vec<&str>,
) -> Result<Instruction, ProgramError> {
    let (challenge_pda, _) =
        Challenge::shank_pda(&challenge_id(), &creator, &id);
    let solutions = hash_solutions(&solutions);

    let ix = Instruction {
        program_id: challenge_id(),
        accounts: vec![
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(creator, true),
            AccountMeta::new(challenge_pda, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: ChallengeInstruction::AddSolutions { id, solutions }
            .try_to_vec()?,
    };

    Ok(ix)
}

// -----------------
// Start Challenge
// -----------------
pub fn start_challenge(
    creator: Pubkey,
    id: String,
) -> Result<Instruction, ProgramError> {
    let (challenge_pda, _) =
        Challenge::shank_pda(&challenge_id(), &creator, &id);

    let ix = Instruction {
        program_id: challenge_id(),
        accounts: vec![
            AccountMeta::new_readonly(creator, true),
            AccountMeta::new(challenge_pda, false),
        ],
        data: ChallengeInstruction::StartChallenge { id }.try_to_vec()?,
    };

    Ok(ix)
}

// -----------------
// Admit Challenger
// -----------------
pub struct AdmitChallengerIx {
    pub challenge_pda: Pubkey,
    pub challenger_pda: Pubkey,
    pub ix: Instruction,
}
pub fn admit_challenger(
    payer: Pubkey,
    creator: Pubkey,
    id: &str,
    challenger: Pubkey,
) -> Result<AdmitChallengerIx, ProgramError> {
    let (challenge_pda, _) =
        Challenge::shank_pda(&challenge_id(), &creator, id);
    let (challenger_pda, _) =
        Challenger::shank_pda(&challenge_id(), &challenge_pda, &challenger);

    let ix = Instruction {
        program_id: challenge_id(),
        accounts: vec![
            AccountMeta::new(payer, true),
            AccountMeta::new(creator, false),
            AccountMeta::new_readonly(challenge_pda, false),
            AccountMeta::new_readonly(challenger, false),
            AccountMeta::new(challenger_pda, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: ChallengeInstruction::AdmitChallenger { challenge_pda }
            .try_to_vec()?,
    };

    Ok(AdmitChallengerIx {
        challenge_pda,
        challenger_pda,
        ix,
    })
}

#[derive(Debug)]
struct RedeemAccounts {
    payer: AccountMeta,
    challenge_pda: AccountMeta,
    challenger: AccountMeta,
    challenger_pda: AccountMeta,
    redeem_pda: AccountMeta,
    redeem_ata: AccountMeta,
    spl_token_program: AccountMeta,
    spl_associated_token_program: AccountMeta,
    system_program: AccountMeta,
}

// -----------------
// Redeem by providing solution
// -----------------
/// Attempts to redeem by providing a solution.
///
/// * [payer]: pays for the transaction and is usually the challenger
/// * [creator]: the authority managing the challenge
/// * [id]: unique id used when creating the challenge
/// * [challenger]: the  account attempting to redeem by providing the solution
/// * [solution]: solutions to be added in clear text, they are encoded via `sha256(solution)`
///   before being passed to the challenge
pub fn redeem(
    payer: Pubkey,
    creator: Pubkey,
    id: &str,
    challenger: Pubkey,
    solution: &str,
) -> Result<Instruction, ProgramError> {
    let challenger_sends = hash_solution_challenger_sends(solution);

    let (challenge_pda, _) =
        Challenge::shank_pda(&challenge_id(), &creator, id);
    let (challenger_pda, _) =
        Challenger::shank_pda(&challenge_id(), &challenge_pda, &challenger);
    let redeem = Redeem::new(challenge_pda);
    let redeem_ata = redeem.ata(&challenger);

    let accounts = RedeemAccounts {
        payer: AccountMeta::new(payer, true),
        challenge_pda: AccountMeta::new(challenge_pda, false),
        challenger: AccountMeta::new_readonly(challenger, true),
        challenger_pda: AccountMeta::new(challenger_pda, false),
        redeem_pda: AccountMeta::new(redeem.pda().0, false),
        redeem_ata: AccountMeta::new(redeem_ata, false),
        spl_token_program: AccountMeta::new_readonly(spl_token::id(), false),
        spl_associated_token_program: AccountMeta::new_readonly(
            spl_associated_token_account::id(),
            false,
        ),
        system_program: AccountMeta::new_readonly(system_program::id(), false),
    };

    let ix = Instruction {
        program_id: challenge_id(),
        accounts: vec![
            accounts.payer,
            accounts.challenge_pda,
            // challenger
            accounts.challenger,
            accounts.challenger_pda,
            // redeem
            accounts.redeem_pda,
            accounts.redeem_ata,
            // programs
            accounts.spl_token_program,
            accounts.spl_associated_token_program,
            accounts.system_program,
        ],
        data: ChallengeInstruction::Redeem {
            solution: challenger_sends,
        }
        .try_to_vec()?,
    };

    Ok(ix)
}
