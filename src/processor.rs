use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    hash::HASH_BYTES,
    msg,
    pubkey::Pubkey,
};

use crate::{
    challenge_id, check_id,
    ixs::ChallengeInstruction,
    state::Challenge,
    utils::{
        allocate_account_and_assign_owner, assert_max_solutions, assert_pda,
        AllocateAndAssignAccountArgs,
    },
};

pub fn process<'a>(
    program_id: &'a Pubkey,
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
            max_solutions,
        } => {
            let max_solutions = max_solutions.unwrap_or(solutions.len() as u8);
            process_create_challenge(
                program_id,
                accounts,
                admit_cost,
                tries_per_admit,
                redeem,
                solutions,
                max_solutions,
            )
        }
        AddSolutions { solutions: _ } => todo!(),
    }
}

// -----------------
// Create Challenge
// -----------------
fn process_create_challenge<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    admit_cost: u64,
    tries_per_admit: u8,
    redeem: Pubkey,
    solutions: Vec<[u8; HASH_BYTES]>,
    max_solutions: u8,
) -> ProgramResult {
    msg!("IX: create challenge");

    // TODO(thlorenz): assert they arent' < solutions.len()
    assert_max_solutions(max_solutions, solutions.len())?;

    // TODO(thlorenz): think about if we need to ensure that we don't allow
    // pre-initialized accounts.
    // Should not be an issue and would also fail when trying to create the
    // account again.

    let account_info_iter = &mut accounts.iter();
    let payer_info = next_account_info(account_info_iter)?;
    let creator_info = next_account_info(account_info_iter)?;
    let challenge_pda_info = next_account_info(account_info_iter)?;

    let (pda, bump) = Challenge::shank_pda(&challenge_id(), creator_info.key);
    assert_pda(
        challenge_pda_info.key,
        &pda,
        "PDA for the challenge for this creator is incorrect",
    )?;

    let bump_arr = [bump];
    let seeds = Challenge::shank_seeds_with_bump(creator_info.key, &bump_arr);

    let size = Challenge::size(max_solutions);
    allocate_account_and_assign_owner(AllocateAndAssignAccountArgs {
        payer_info,
        account_info: challenge_pda_info,
        owner: program_id,
        signer_seeds: &seeds,
        size,
    })?;

    let challenge = Challenge {
        authority: *creator_info.key,
        admit_cost,
        tries_per_admit,
        redeem,
        solving: 0,
        solutions,
    };

    challenge.serialize(
        &mut &mut challenge_pda_info.try_borrow_mut_data()?.as_mut(),
    )?;

    msg!("Challenge account created and initialized ");

    Ok(())
}
