use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};

use crate::{
    challenge_id, check_id,
    ixs::ChallengeInstruction,
    state::{Challenge, StateFromPdaAccountValue},
    utils::{
        allocate_account_and_assign_owner, assert_account_has_no_data,
        assert_adding_non_empty, assert_can_add_solutions,
        assert_has_solutions, assert_keys_equal,
        assert_max_supported_solutions, assert_not_started, reallocate_account,
        AllocateAndAssignAccountArgs, ReallocateAccountArgs,
    },
    Solution,
};

// -----------------
// Processor Entry
// -----------------
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
            id,
            admit_cost,
            tries_per_admit,
            redeem,
            solutions,
        } => process_create_challenge(
            program_id,
            accounts,
            id,
            admit_cost,
            tries_per_admit,
            redeem,
            solutions,
        ),
        AddSolutions { id, solutions } => {
            process_add_solutions(program_id, accounts, id, solutions)
        }
        StartChallenge { id } => {
            process_start_challenge(program_id, accounts, id)
        }
    }
}

// -----------------
// Create Challenge
// -----------------
fn process_create_challenge<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    id: String,
    admit_cost: u64,
    tries_per_admit: u8,
    redeem: Pubkey,
    solutions: Vec<Solution>,
) -> ProgramResult {
    msg!("IX: create challenge");

    assert_keys_equal(program_id, &challenge_id(), || {
        format!(
            "Provided program id ({}) does not match this program's id ({})",
            program_id,
            challenge_id()
        )
    })?;

    assert_max_supported_solutions(&solutions)?;

    // TODO(thlorenz): think about if we need to ensure that we don't allow
    // pre-initialized accounts.
    // Should not be an issue and would also fail when trying to create the
    // account again.

    let account_info_iter = &mut accounts.iter();
    let payer_info = next_account_info(account_info_iter)?;
    let creator_info = next_account_info(account_info_iter)?;
    let challenge_pda_info = next_account_info(account_info_iter)?;

    let (pda, bump) =
        Challenge::shank_pda(&challenge_id(), creator_info.key, &id);
    assert_keys_equal(challenge_pda_info.key, &pda, || {
        format!(
            "PDA for the challenge for creator ({}) and id ({}) is incorrect",
            creator_info.key, id
        )
    })?;
    assert_account_has_no_data(challenge_pda_info)?;

    let bump_arr = [bump];
    let seeds =
        Challenge::shank_seeds_with_bump(creator_info.key, &id, &bump_arr);

    let size = Challenge::needed_size(&solutions, &id);
    allocate_account_and_assign_owner(AllocateAndAssignAccountArgs {
        payer_info,
        account_info: challenge_pda_info,
        owner: program_id,
        signer_seeds: &seeds,
        size,
    })?;

    let challenge = Challenge {
        authority: *creator_info.key,
        id,
        started: false,
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

// -----------------
// Add Solutions
// -----------------
fn process_add_solutions<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    id: String,
    extra_solutions: Vec<Solution>,
) -> ProgramResult {
    msg!("IX: add solutions");

    assert_keys_equal(program_id, &challenge_id(), || {
        format!(
            "Provided program id ({}) does not match this program's id ({})",
            program_id,
            challenge_id()
        )
    })?;
    assert_adding_non_empty(&extra_solutions)?;

    let account_info_iter = &mut accounts.iter();
    let payer_info = next_account_info(account_info_iter)?;
    let creator_info = next_account_info(account_info_iter)?;
    let challenge_pda_info = next_account_info(account_info_iter)?;

    let StateFromPdaAccountValue::<Challenge> {
        state: mut challenge,
        ..
    } = Challenge::account_state_verifying_creator(
        challenge_pda_info,
        creator_info,
        &id,
    )?;

    // 1. append solutions
    assert_can_add_solutions(&challenge.solutions, &extra_solutions)?;
    challenge.solutions.extend(extra_solutions);

    // 2. reallocate account to fit extra solutions, including upping lamports to stay rent excempt
    let size = challenge.size();
    reallocate_account(ReallocateAccountArgs {
        payer_info,
        account_info: challenge_pda_info,
        new_size: size,
        zero_init: false,
    })?;

    challenge.serialize(
        &mut &mut challenge_pda_info.try_borrow_mut_data()?.as_mut(),
    )?;

    Ok(())
}

// -----------------
// Start Challenge
// -----------------
fn process_start_challenge(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    id: String,
) -> ProgramResult {
    msg!("IX: start challenge");

    assert_keys_equal(program_id, &challenge_id(), || {
        format!(
            "Provided program id ({}) does not match this program's id ({})",
            program_id,
            challenge_id()
        )
    })?;

    let account_info_iter = &mut accounts.iter();
    let creator_info = next_account_info(account_info_iter)?;
    let challenge_pda_info = next_account_info(account_info_iter)?;

    let StateFromPdaAccountValue::<Challenge> {
        state: mut challenge,
        ..
    } = Challenge::account_state_verifying_creator(
        challenge_pda_info,
        creator_info,
        &id,
    )?;

    assert_not_started(&challenge)?;
    assert_has_solutions(&challenge, "be started")?;

    challenge.started = true;
    challenge.serialize(
        &mut &mut challenge_pda_info.try_borrow_mut_data()?.as_mut(),
    )?;

    Ok(())
}

/* TODO(thlorenz): @@@ get to this once we finish with the trait stuff
// -----------------
// Admit Challenger
// -----------------
fn process_admit_challenger(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    challenge_pda: Pubkey,
) -> ProgramResult {
    msg!("IX: admit challenger");

    assert_keys_equal(program_id, &challenge_id(), || {
        format!(
            "Provided program id ({}) does not match this program's id ({})",
            program_id,
            challenge_id()
        )
    })?;

    let account_info_iter = &mut accounts.iter();
    let creator_info = next_account_info(account_info_iter)?;
    let challenge_pda_info = next_account_info(account_info_iter)?;
    let challenger_info = next_account_info(account_info_iter)?;
    let challenger_pda_info = next_account_info(account_info_iter)?;

    assert_keys_equal(&challenge_pda, challenge_pda_info.key, || {
        format!(
                "Provided challenge pda ({}) key does not match the provided PDA account {})",
                challenge_pda, challenge_pda_info.key
            )
    })?;

    let PdaAccountInfo {
        account: mut challenge,
        ..
    } = Challenge::mutable_from_data(challenge_pda_info)?;

    assert_started(&challenge)?;

    challenge.serialize(
        &mut &mut challenge_pda_info.try_borrow_mut_data()?.as_mut(),
    )?;

    Ok(())
}
*/
