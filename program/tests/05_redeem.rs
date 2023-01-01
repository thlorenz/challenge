#![cfg(feature = "test-sbf")]

use assert_matches::assert_matches;

use challenge::{
    ixs,
    state::{Challenge, Challenger, HasPda, Redeem},
    utils::hash_solutions,
};

use solana_program::pubkey::Pubkey;
use solana_program_test::*;

#[allow(unused)]
use crate::utils::dump_account;
use crate::utils::{
    add_mint_to_redeem, add_pda_account, verify_minted_when_redeeming,
};
use solana_sdk::{
    signature::Keypair, signer::Signer, transaction::Transaction,
};

use crate::utils::{get_deserialized, program_test};

mod utils;

const ID: &str = "challenge-id";
const ADMIT_COST: u64 = 200;
const TRIES_PER_ADMIT: u8 = 11;

async fn admitted_challenger_redeems_with(
    context: &mut ProgramTestContext,
    challenge: &Challenge,
    creator: Pubkey,
    solution: &str,
) -> Challenger {
    let challenger_pair = Keypair::new();
    let challenger_key = challenger_pair.pubkey();

    let challenger = Challenger {
        authority: challenger_key,
        challenge_pda: challenge.pda().0,
        tries_remaining: TRIES_PER_ADMIT,
        redeemed: false,
    };
    add_pda_account(context, &challenger);

    let ix = ixs::redeem(
        context.payer.pubkey(),
        creator,
        ID,
        challenger_key,
        solution,
    )
    .expect("failed to create instruction");

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &challenger_pair],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(tx)
        .await
        .expect("Failed to redeem");

    challenger
}

#[tokio::test]
async fn redeem_for_valid_challenge_and_two_challengers_with_correct_solution()
{
    let mut context = program_test().start_with_context().await;
    let creator = Pubkey::new_unique();

    let redeem = Redeem::for_challenge_with(&creator, ID);

    let solutions = hash_solutions(&["hello", "world"]);
    let challenge = &Challenge {
        authority: creator,
        id: ID.to_string(),
        started: true,
        finished: false,
        admit_cost: ADMIT_COST,
        tries_per_admit: TRIES_PER_ADMIT,
        redeem: redeem.pda().0,
        solving: 0,
        solutions,
    };
    add_pda_account(&mut context, challenge);
    add_mint_to_redeem(&mut context, &redeem);

    let (mint_pda, _) = redeem.pda();
    // First challenger solves the challenge
    // NOTE: that we're not sure yet if we allow the same challaneger to solve
    // multiple times. Therefore we simulate two different challengers here
    {
        let challenger = admitted_challenger_redeems_with(
            &mut context,
            challenge,
            creator,
            "hello",
        )
        .await;

        let (_, challenger_value) =
            get_deserialized::<Challenger>(&mut context, &challenger.pda().0)
                .await;

        assert_matches!(
            challenger_value,
            Challenger {
                authority: _,
                challenge_pda: _,
                tries_remaining,
                redeemed: true,
            } => {
                assert_eq!(tries_remaining, TRIES_PER_ADMIT - 1);
            }
        );

        assert_matches!(
            get_deserialized::<Challenge>(&mut context, &challenge.pda().0)
                .await
                .1,
            Challenge {
                authority: _,
                id: _,
                started: true,
                finished: false,
                admit_cost: ADMIT_COST,
                solving: 1,
                solutions: _,
                tries_per_admit: TRIES_PER_ADMIT,
                redeem: _,
            }
        );

        verify_minted_when_redeeming(
            &mut context,
            mint_pda,
            1,
            &redeem,
            &challenger,
        )
        .await;
    }
    // Second challenger solves the challenge which finishes it
    {
        let challenger = admitted_challenger_redeems_with(
            &mut context,
            challenge,
            creator,
            "world",
        )
        .await;

        let (_, challenger_value) =
            get_deserialized::<Challenger>(&mut context, &challenger.pda().0)
                .await;

        assert_matches!(
            challenger_value,
            Challenger {
                authority: _,
                challenge_pda: _,
                tries_remaining,
                redeemed: true,
            } => {
                assert_eq!(tries_remaining, TRIES_PER_ADMIT - 1);
            }
        );

        assert_matches!(
            get_deserialized::<Challenge>(&mut context, &challenge.pda().0)
                .await
                .1,
            Challenge {
                authority: _,
                id: _,
                started: true,
                finished: true,
                admit_cost: ADMIT_COST,
                solving: 2,
                solutions: _,
                tries_per_admit: TRIES_PER_ADMIT,
                redeem: _,
            }
        );
        verify_minted_when_redeeming(
            &mut context,
            mint_pda,
            2,
            &redeem,
            &challenger,
        )
        .await;
    }
}

// -----------------
// Error Cases
//-----------------
// TODO(thlorenz): once we have minting on redeem working test the following error cases
// - redeeming with incorrect solution
// - redeeming when challenge is finished
// - redeeming when challenge was not started yet
// - redeeming with challenger that never was admitted
// - redeeming with challenger that was admitted but ran out of tries
// - redeeming with challenger that was admitted but already redeemed (possibly need a config on
//   the challenge if multiple redeems are allowed or not)
//
