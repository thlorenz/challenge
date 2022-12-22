#![cfg(feature = "test-sbf")]

use crate::utils::{get_deserialized, hash_solution};
use assert_matches::assert_matches;
use challenge::{challenge_id, ixs, state::Challenge};
use solana_program::pubkey::Pubkey;
use solana_program_test::*;

use solana_sdk::{signer::Signer, transaction::Transaction};

#[allow(unused)]
use crate::utils::dump_account;
use crate::utils::{ixs_custom, program_test};

mod utils;

const ID: &str = "challenge-id";

#[tokio::test]
async fn create_challenge_without_solutions() {
    let mut context = program_test().start_with_context().await;
    let creator = context.payer.pubkey();
    let redeem = Pubkey::new_unique();

    let ix = ixs::create_challenge(
        creator,
        creator,
        ID.to_string(),
        1000,
        1,
        redeem,
        vec![],
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

    let (challenge_pda, _) =
        Challenge::shank_pda(&challenge_id(), &creator, ID);
    let (acc, value) =
        get_deserialized::<Challenge>(&mut context, &challenge_pda).await;

    assert_matches!(
        value,
        Challenge {
            authority,
            id,
            admit_cost: 1000,
            tries_per_admit: 1,
            redeem: r,
            solving: 0,
            solutions,
        } => {
            assert_eq!(&authority, &creator);
            assert_eq!(id, ID);
            assert_eq!(&r, &redeem);
            assert!(solutions.is_empty());
            assert_eq!(acc.data.len(), Challenge::needed_size(&solutions, ID));
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
        ID.to_string(),
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
    let (challenge_pda, _) =
        Challenge::shank_pda(&challenge_id(), &creator, ID);
    let (acc, value) =
        get_deserialized::<Challenge>(&mut context, &challenge_pda).await;

    assert_matches!(
        value,
        Challenge {
            authority,
            id,
            admit_cost: 1000,
            tries_per_admit: 1,
            redeem: r,
            solving: 0,
            solutions,
        } => {
            assert_eq!(&authority, &creator);
            assert_eq!(id, ID);
            assert_eq!(&r, &redeem);
            assert_eq!(solutions.len(), 2);
            assert_eq!(solutions[0], hash_solution("hello"));
            assert_eq!(solutions[1], hash_solution("world"));
            assert_eq!(acc.data.len(), Challenge::needed_size(&solutions, ID));
        }
    );
}

#[tokio::test]
async fn create_two_challenges_same_creator_different_id() {
    let mut context = program_test().start_with_context().await;
    let creator = context.payer.pubkey();
    let redeem = Pubkey::new_unique();

    let fst_id = "challenge-id-1";
    let snd_id = "challenge-id-2";

    // Create first Challenge
    {
        let ix = ixs::create_challenge(
            creator,
            creator,
            fst_id.to_string(),
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
    }

    // Create second Challenge
    {
        let ix = ixs::create_challenge(
            creator,
            creator,
            snd_id.to_string(),
            2000,
            2,
            redeem,
            vec!["hola", "mundo"],
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
    }

    // Check first Challenge
    {
        let (challenge_pda, _) =
            Challenge::shank_pda(&challenge_id(), &creator, fst_id);
        let (acc, value) =
            get_deserialized::<Challenge>(&mut context, &challenge_pda).await;

        assert_matches!(
            value,
            Challenge {
                authority,
                id,
                admit_cost: 1000,
                tries_per_admit: 1,
                redeem: r,
                solving: 0,
                solutions,
            } => {
                assert_eq!(&authority, &creator);
                assert_eq!(id, fst_id);
                assert_eq!(&r, &redeem);
                assert_eq!(solutions.len(), 2);
                assert_eq!(solutions[0], hash_solution("hello"));
                assert_eq!(solutions[1], hash_solution("world"));
                assert_eq!(acc.data.len(), Challenge::needed_size(&solutions, &id));
            }
        );
    }
    //
    // Check second Challenge
    {
        let (challenge_pda, _) =
            Challenge::shank_pda(&challenge_id(), &creator, snd_id);
        let (acc, value) =
            get_deserialized::<Challenge>(&mut context, &challenge_pda).await;

        assert_matches!(
            value,
            Challenge {
                authority,
                id,
                admit_cost: 2000,
                tries_per_admit: 2,
                redeem: r,
                solving: 0,
                solutions,
            } => {
                assert_eq!(&authority, &creator);
                assert_eq!(id, snd_id);
                assert_eq!(&r, &redeem);
                assert_eq!(solutions.len(), 2);
                assert_eq!(solutions[0], hash_solution("hola"));
                assert_eq!(solutions[1], hash_solution("mundo"));
                assert_eq!(acc.data.len(), Challenge::needed_size(&solutions, &id));
            }
        );
    }
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
        ID.to_string(),
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

#[tokio::test]
#[should_panic]
async fn create_two_challenges_same_creator_same_id() {
    let mut context = program_test().start_with_context().await;
    let creator = context.payer.pubkey();
    let redeem = Pubkey::new_unique();

    let fst_id = "challenge-id-1";
    let snd_id = "challenge-id-1";

    // Create first Challenge
    {
        let ix = ixs::create_challenge(
            creator,
            creator,
            fst_id.to_string(),
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
    }

    // Try to reate second Challenge (this fails)
    {
        let ix = ixs::create_challenge(
            creator,
            creator,
            snd_id.to_string(),
            2000,
            2,
            redeem,
            vec!["hola", "mundo"],
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
    }
}
