#![cfg(feature = "test-sbf")]

use challenge::{ixs, state::Challenge, utils::hash_solutions};
use solana_program::pubkey::Pubkey;
use solana_program_test::*;

use solana_sdk::{signer::Signer, transaction::Transaction};
use utils::add_challenge_account;

use crate::utils::program_test;

mod utils;

fn add_challenge_with_solutions(
    context: &mut ProgramTestContext,
    solutions: Vec<&str>,
) {
    let solutions = hash_solutions(&solutions);
    add_challenge_account(
        context,
        Challenge {
            authority: context.payer.pubkey(),
            admit_cost: 200,
            tries_per_admit: 1,
            redeem: Pubkey::new_unique(),
            solving: 0,
            solutions,
        },
    );
}

#[tokio::test]
async fn add_solutions_to_empty_solutions_push() {
    let mut context = program_test().start_with_context().await;
    add_challenge_with_solutions(&mut context, vec![]);

    let solutions = vec!["hello", "world"];

    let ix = ixs::add_solutions(context.payer.pubkey(), solutions, None)
        .expect("failed to create instruction");
    let _tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    // TODO(thlorenz): Work on this once we have realloc implemented
    /*
    context
        .banks_client
        .process_transaction(tx)
        .await
        .expect("Failed add solutions");

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
        }
    );

    assert_eq!(acc.data.len(), Challenge::size(2));
    */
}
