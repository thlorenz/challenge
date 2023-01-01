#![cfg(feature = "test-sbf")]

use assert_matches::assert_matches;

use borsh::BorshSerialize;
use challenge::{
    challenge_id,
    ixs::{self, ChallengeInstruction},
    state::Challenge,
};

use solana_program::instruction::{AccountMeta, Instruction};
use solana_program_test::*;

use solana_sdk::{
    signature::Keypair, signer::Signer, transaction::Transaction,
};
use utils::{
    add_challenge_with_solutions, add_started_challenge_with_solutions,
};

use crate::utils::{get_deserialized, hash_solution, program_test};

mod utils;
const ID: &str = "challenge-id";

#[tokio::test]
async fn start_challenge_that_is_not_started_and_has_solutions() {
    let mut context = program_test().start_with_context().await;
    let creator = context.payer.pubkey();
    add_challenge_with_solutions(
        &mut context,
        ID,
        vec!["hello", "world"],
        None,
    );

    let (challenge_pda, _) =
        Challenge::shank_pda(&challenge_id(), &context.payer.pubkey(), ID);

    let ix = ixs::start_challenge(creator, ID.to_string())
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
        .expect("Failed to start challenge");

    let (_, value) =
        get_deserialized::<Challenge>(&mut context, &challenge_pda).await;

    assert_matches!(
        value,
        Challenge {
            authority,
            id,
            started: true,
            finished: false,
            admit_cost: 200,
            tries_per_admit: 1,
            redeem: _,
            solving: 0,
            solutions,
        } => {
            assert_eq!(&authority, &creator);
            assert_eq!(id, ID);
            assert_eq!(solutions.len(), 2);
            assert_eq!(solutions[0], hash_solution("hello"));
            assert_eq!(solutions[1], hash_solution("world"));
        }
    );
}

// -----------------
// Error Cases
// -----------------
#[tokio::test]
#[should_panic]
async fn start_challenge_that_is_already_started_and_has_solutions() {
    let mut context = program_test().start_with_context().await;
    let creator = context.payer.pubkey();
    add_started_challenge_with_solutions(
        &mut context,
        ID,
        vec!["hello", "world"],
        None,
    );

    let ix = ixs::start_challenge(creator, ID.to_string())
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
        .expect("Failed start challenge");
}

#[tokio::test]
#[should_panic]
async fn start_challenge_that_is_not_started_but_has_no_solutions() {
    let mut context = program_test().start_with_context().await;
    let creator = context.payer.pubkey();
    add_challenge_with_solutions(&mut context, ID, vec![], None);

    let ix = ixs::start_challenge(creator, ID.to_string())
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
        .expect("Failed start challenge");
}

#[tokio::test]
#[should_panic]
async fn start_challenge_with_creator_not_signer() {
    let mut context = program_test().start_with_context().await;
    let creator_pair = Keypair::new();
    let creator = creator_pair.pubkey();
    add_challenge_with_solutions(
        &mut context,
        ID,
        vec!["hello", "world"],
        Some(creator),
    );

    let ix = {
        let (challenge_pda, _) =
            Challenge::shank_pda(&challenge_id(), &creator, ID);
        Instruction {
            program_id: challenge_id(),
            accounts: vec![
                AccountMeta::new_readonly(creator, false),
                AccountMeta::new(challenge_pda, false),
            ],
            data: ChallengeInstruction::StartChallenge { id: ID.to_string() }
                .try_to_vec()
                .expect("failed to create custom instruction"),
        }
    };

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
        .expect("Failed to start challenge");
}
