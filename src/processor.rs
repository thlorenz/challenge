use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    hash::HASH_BYTES,
    msg,
    pubkey::Pubkey,
};

use crate::{
    challenge_id, check_id, ixs::ChallengeInstruction, state::Challenge,
    utils::assert_pda,
};

pub fn process<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    check_id(program_id);

    let instruction = ChallengeInstruction::try_from_slice(instruction_data)?;

    use ChallengeInstruction::*;
    match instruction {
        CreateChallenge {
            admit_cost,
            tries_per_admit,
            redeem,
            solutions,
        } => process_create_challenge(
            program_id,
            accounts,
            admit_cost,
            tries_per_admit,
            redeem,
            solutions,
        ),
        AddSolutions { solutions: _ } => todo!(),
    }
}

// -----------------
// Create Challenge
// -----------------
fn process_create_challenge<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    admit_cost: u64,
    tries_per_admi: u64,
    redeem: Pubkey,
    solutions: Option<Vec<[u8; HASH_BYTES]>>,
) -> ProgramResult {
    msg!("IX: create challenge");

    let account_info_iter = &mut accounts.iter();
    let creator_info = next_account_info(account_info_iter)?;
    let challenge_pda_info = next_account_info(account_info_iter)?;

    let (pda, _) = Challenge::shank_pda(&challenge_id(), creator_info.key);
    assert_pda(
        challenge_pda_info.key,
        &pda,
        "PDA for the challenge for this creator is incorrect",
    )?;

    Ok(())
}
