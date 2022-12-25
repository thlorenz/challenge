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
    state::{
        Challenge, Challenger, HasSize, StateFromPdaAccountValue,
        TryStateFromAccount,
    },
    utils::{
        allocate_account_and_assign_owner, assert_account_does_not_exist,
        assert_account_has_no_data, assert_adding_non_empty,
        assert_can_add_solutions, assert_challenger_has_tries_remaining,
        assert_has_solution, assert_has_solutions, assert_is_signer,
        assert_keys_equal, assert_max_supported_solutions, assert_not_finished,
        assert_not_started, assert_started, reallocate_account,
        transfer_lamports, AllocateAndAssignAccountArgs, ReallocateAccountArgs,
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
        AdmitChallenger { challenge_pda } => {
            process_admit_challenger(program_id, accounts, challenge_pda)
        }
        Redeem { solution } => process_redeem(program_id, accounts, solution),
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

    // TODO(thlorenz): make sure that the creator is rent excempt, as otherwise
    // he won't be able to receive funds from admitted challengers
    // Alternatively we can make sure of that here
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
        finished: false,
        admit_cost,
        tries_per_admit,
        redeem,
        solving: 0,
        solutions,
    };

    challenge.serialize(
        &mut &mut challenge_pda_info.try_borrow_mut_data()?.as_mut(),
    )?;

    msg!("Challenge account created and initialized");

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

    // TODO(thlorenz): @@@ update finished if we now can solve solutios again

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

// -----------------
// Admit Challenger
// -----------------
fn process_admit_challenger<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
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
    let payer_info = next_account_info(account_info_iter)?;
    let creator_info = next_account_info(account_info_iter)?;
    let challenge_pda_info = next_account_info(account_info_iter)?;
    let challenger_info = next_account_info(account_info_iter)?;
    let challenger_pda_info = next_account_info(account_info_iter)?;

    assert_keys_equal(challenge_pda_info.key, &challenge_pda, || {
        format!(
            "Provided challenge pda ({}) does not match the PDA account ({}) provided in the instruction",
            challenge_pda, challenge_pda_info.key
        )
    })?;
    assert_account_does_not_exist(challenger_pda_info, "challenger PDA")?;

    let challenge: Challenge = challenge_pda_info.try_state_from_account()?;
    assert_started(&challenge)?;
    assert_not_finished(&challenge)?;

    // 1. create challenger account
    let (pda, bump) = Challenger::shank_pda(
        &challenge_id(),
        &challenge_pda,
        challenger_info.key,
    );

    assert_keys_equal(challenger_pda_info.key, &pda, || {
        format!(
            "PDA account ({}) provided for the challenger is not a valid for this challenge",
            challenger_pda_info.key
        )
    })?;

    let bump_arr = [bump];
    let seeds = Challenger::shank_seeds_with_bump(
        &challenge_pda,
        challenger_info.key,
        &bump_arr,
    );

    let size = Challenger::size();
    allocate_account_and_assign_owner(AllocateAndAssignAccountArgs {
        payer_info,
        account_info: challenger_pda_info,
        owner: program_id,
        signer_seeds: &seeds,
        size,
    })?;

    // 2. initialize challenger account using data from the challenge
    let challenger = Challenger {
        authority: *challenger_info.key,
        challenge_pda,
        tries_remaining: challenge.tries_per_admit,
        redeemed: false,
    };

    challenger.serialize(
        &mut &mut challenger_pda_info.try_borrow_mut_data()?.as_mut(),
    )?;

    // 3. transfer admit cost to creator account
    transfer_lamports(payer_info, creator_info, challenge.admit_cost)?;

    Ok(())
}

// -----------------
// Redeem by proposing solution
// -----------------
fn process_redeem<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    solution: Solution,
) -> ProgramResult {
    msg!("IX: redeem");

    assert_keys_equal(program_id, &challenge_id(), || {
        format!(
            "Provided program id ({}) does not match this program's id ({})",
            program_id,
            challenge_id()
        )
    })?;

    let account_info_iter = &mut accounts.iter();
    let payer_info = next_account_info(account_info_iter)?;
    let challenge_pda_info = next_account_info(account_info_iter)?;
    let challenger_info = next_account_info(account_info_iter)?;
    let challenger_pda_info = next_account_info(account_info_iter)?;

    assert_is_signer(payer_info, "payer")?;
    assert_is_signer(challenger_info, "challenger")?;

    let mut challenger: Challenger =
        challenger_pda_info.try_state_from_account()?;

    assert_keys_equal(
        &challenger.challenge_pda,
        challenge_pda_info.key,
        || {
            format!(
            "Challenge pda ({}) of provided callenger does not match the PDA account ({}) for which you are trying to redeem",
            &challenger.challenge_pda, challenge_pda_info.key
        )
        },
    )?;

    let challenge: Challenge = challenge_pda_info.try_state_from_account()?;
    // TODO(thlorenz): Technically the challenger would not have been admitted if the challenge
    // wasn't already started, so might not need this check
    assert_started(&challenge)?;
    assert_not_finished(&challenge)?;

    assert_challenger_has_tries_remaining(&challenger)?;
    assert_has_solution(&challenge)?;

    if challenge.is_solution_correct(&solution) {
        // TODO(thlorenz): @@@ increment solving
        // TODO(thlorenz): @@@ update finished if out of solutions
        // TODO(thlorenz): mint_token_to_player but first need to init the token when challenge is
        msg!("TODO: mint token ({})", challenge.redeem);
    } else {
        challenger.tries_remaining -= 1;
        msg!("Provides solution was incorrect");
    }

    // created

    Ok(())
}
