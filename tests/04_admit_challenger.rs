#![cfg(feature = "test-sbf")]

use assert_matches::assert_matches;

use challenge::{
    ixs::{self, AdmitChallengerIx},
    state::{Challenge, Challenger, HasPda},
    utils::hash_solutions,
};

use solana_program::pubkey::Pubkey;
use solana_program_test::*;

#[allow(unused)]
use crate::utils::dump_account;
use crate::utils::{add_pda_account, airdrop_rent};
use solana_sdk::{signer::Signer, transaction::Transaction};

use crate::utils::{get_account, get_deserialized, program_test};

mod utils;

const ID: &str = "challenge-id";
const ADMIT_COST: u64 = 200;
const TRIES_PER_ADMIT: u8 = 11;

#[tokio::test]
async fn admit_challenger_to_started_challenge() {
    let mut context = program_test().start_with_context().await;

    let creator = Pubkey::new_unique();
    let creator_lamports = airdrop_rent(&mut context, &creator, 0).await;

    let payer = context.payer.pubkey();
    let challenger = Pubkey::new_unique();

    let solutions = hash_solutions(&["hello", "world"]);
    add_pda_account(
        &mut context,
        &Challenge {
            authority: creator,
            id: ID.to_string(),
            started: true,
            admit_cost: ADMIT_COST,
            tries_per_admit: TRIES_PER_ADMIT,
            redeem: Pubkey::new_unique(),
            solving: 0,
            solutions,
        },
    );

    let AdmitChallengerIx {
        ix,
        challenge_pda,
        challenger_pda,
    } = ixs::admit_challenger(payer, creator, ID, challenger)
        .expect("failed to create instruction");

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(tx)
        .await
        .expect("Failed to admit challenger");

    // Verify that the challenger account was created correctly
    let (acc, value) =
        get_deserialized::<Challenger>(&mut context, &challenger_pda).await;

    assert_matches!(
        value,
        Challenger {
            authority,
            challenge_pda: cpda,
            tries_remaining: TRIES_PER_ADMIT,
            redeemed: false,
        } => {
            assert_eq!(&authority, &challenger);
            assert_eq!(&cpda, &challenge_pda);
            assert_eq!(acc.data.len(), Challenger::size());
            assert!(acc.lamports >= 1_350_000);
        }
    );

    // Verify that creator was paid the admit fee
    let creator_acc = get_account(&mut context, &creator).await;
    assert_eq!(
        creator_acc.lamports,
        creator_lamports + ADMIT_COST,
        "creator should have received admit cost"
    );
}

// -----------------
// Error Cases
// -----------------

#[tokio::test]
#[should_panic]
async fn admit_challenger_trying_twice_for_same_challenge() {
    let mut context = program_test().start_with_context().await;

    let creator = Pubkey::new_unique();
    airdrop_rent(&mut context, &creator, 0).await;

    let payer = context.payer.pubkey();
    let challenger = Pubkey::new_unique();

    let solutions = hash_solutions(&["hello", "world"]);

    let challenge = Challenge {
        authority: creator,
        id: ID.to_string(),
        started: true,
        admit_cost: ADMIT_COST,
        tries_per_admit: TRIES_PER_ADMIT,
        redeem: Pubkey::new_unique(),
        solving: 0,
        solutions,
    };

    let (challenge_pda, _) = challenge.pda();
    add_pda_account(&mut context, &challenge);

    add_pda_account(
        &mut context,
        &Challenger {
            authority: challenger,
            challenge_pda,
            tries_remaining: TRIES_PER_ADMIT,
            redeemed: false,
        },
    );

    let AdmitChallengerIx { ix, .. } =
        ixs::admit_challenger(payer, creator, ID, challenger)
            .expect("failed to create instruction");

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(tx)
        .await
        .expect("Failed to admit challenger");
}

#[tokio::test]
#[should_panic]
async fn admit_challenger_to_not_yet_started_challenge() {
    let mut context = program_test().start_with_context().await;

    let creator = Pubkey::new_unique();
    airdrop_rent(&mut context, &creator, 0).await;

    let payer = context.payer.pubkey();
    let challenger = Pubkey::new_unique();

    let solutions = hash_solutions(&["hello", "world"]);

    let challenge = &Challenge {
        authority: creator,
        id: ID.to_string(),
        started: false,
        admit_cost: ADMIT_COST,
        tries_per_admit: TRIES_PER_ADMIT,
        redeem: Pubkey::new_unique(),
        solving: 0,
        solutions,
    };

    add_pda_account(&mut context, challenge);

    let AdmitChallengerIx { ix, .. } =
        ixs::admit_challenger(payer, creator, ID, challenger)
            .expect("failed to create instruction");

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(tx)
        .await
        .expect("Failed to admit challenger");
}

// TODO(thlorenz): there are lots of other invalid cases we should ensure are handled properly
// TODO(thlorenz): Additionally we should put in the extra work to convert the `should_panic` tests
// to perform more specific asserts on the error returned.
