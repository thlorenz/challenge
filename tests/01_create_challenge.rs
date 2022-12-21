#![cfg(feature = "test-sbf")]

use crate::utils::{get_deserialized, hash_solution};
use assert_matches::assert_matches;
use challenge::{challenge_id, ixs, state::Challenge};
use solana_program::pubkey::Pubkey;
use solana_program_test::*;

use solana_sdk::{signer::Signer, transaction::Transaction};

use crate::utils::{ixs_custom, program_test};

mod utils;

#[tokio::test]
async fn create_challenge_without_solutions() {
    let mut context = program_test().start_with_context().await;
    let creator = context.payer.pubkey();
    let redeem = Pubkey::new_unique();

    let ix = ixs::create_challenge(creator, creator, 1000, 1, redeem, vec![])
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
        .expect("Failed create challenge");

    let (challenge_pda, _) = Challenge::shank_pda(&challenge_id(), &creator);
    let (acc, value) =
        get_deserialized::<Challenge>(&mut context, &challenge_pda).await;

    assert_matches!(
        value,
        Challenge {
            authority,
            admit_cost: 1000,
            tries_per_admit: 1,
            redeem: r,
            solving: 0,
            solutions,
        } => {
            assert_eq!(&authority, &creator);
            assert_eq!(&r, &redeem);
            assert!(solutions.is_empty());
            assert_eq!(acc.data.len(), Challenge::needed_size(&solutions));
        }
    );
}

#[tokio::test]
async fn create_challenge_with_two_solutions() {
    let mut context = program_test().start_with_context().await;
    let creator = context.payer.pubkey();
    let redeem = Pubkey::new_unique();

    let ix = ixs::create_challenge(
        creator,
        creator,
        1000,
        1,
        redeem,
        vec!["hello", "world"],
    )
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
        .expect("Failed create challenge");

    // Checks
    let (challenge_pda, _) = Challenge::shank_pda(&challenge_id(), &creator);
    let (acc, value) =
        get_deserialized::<Challenge>(&mut context, &challenge_pda).await;

    assert_matches!(
        value,
        Challenge {
            authority,
            admit_cost: 1000,
            tries_per_admit: 1,
            redeem: r,
            solving: 0,
            solutions,
        } => {
            assert_eq!(&authority, &creator);
            assert_eq!(&r, &redeem);
            assert_eq!(solutions.len(), 2);
            assert_eq!(solutions[0], hash_solution("hello"));
            assert_eq!(solutions[1], hash_solution("world"));
            assert_eq!(acc.data.len(), Challenge::needed_size(&solutions));
        }
    );
}

// -----------------
// Error Cases
// -----------------
#[tokio::test]
#[should_panic]
async fn create_challenge_with_invalid_pda() {
    let mut context = program_test().start_with_context().await;
    let creator = context.payer.pubkey();
    let redeem = Pubkey::new_unique();

    let ix = ixs_custom::create_challenge_with_pda(
        creator,
        creator,
        1000,
        1,
        redeem,
        vec![],
        Pubkey::new_unique(),
    )
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
        .expect("Failed to verify minted token");
}
