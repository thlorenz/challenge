use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh::try_from_slice_unchecked,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};

use crate::{
    challenge_id, check_id,
    ixs::ChallengeInstruction,
    state::Challenge,
    utils::{
        allocate_account_and_assign_owner, assert_account_has_no_data,
        assert_account_is_funded_and_has_data, assert_adding_non_empty,
        assert_can_add_solutions, assert_is_signer, assert_keys_equal,
        assert_max_supported_solutions, reallocate_account,
        AllocateAndAssignAccountArgs, ReallocateAccountArgs,
    },
    Solution,
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
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    id: String,
    extra_solutions: Vec<Solution>,
) -> ProgramResult {
    msg!("IX: add solutions");

    assert_adding_non_empty(&extra_solutions)?;

    let account_info_iter = &mut accounts.iter();
    let payer_info = next_account_info(account_info_iter)?;
    let creator_info = next_account_info(account_info_iter)?;
    let challenge_pda_info = next_account_info(account_info_iter)?;

    let (pda, _bump) =
        Challenge::shank_pda(&challenge_id(), creator_info.key, &id);

    assert_keys_equal(challenge_pda_info.key, &pda, || {
        format!(
            "PDA for the challenge for creator ({}) and id ({}) is incorrect",
            creator_info.key, id
        )
    })?;
    assert_account_is_funded_and_has_data(challenge_pda_info)?;

    let mut challenge = {
        let challenge_data = &challenge_pda_info.try_borrow_data()?;
        try_from_slice_unchecked::<Challenge>(challenge_data)?
    };

    assert_is_signer(creator_info, "creator")?;

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
