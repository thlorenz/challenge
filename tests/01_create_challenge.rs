#![cfg(feature = "test-sbf")]

use challenge::ixs;
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

    let ix = ixs::create_challenge(creator, creator, 1000, 1, redeem, None)
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
        None,
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
