#![cfg(feature = "test-sbf")]

use assert_matches::assert_matches;
use challenge::{challenge_id, ixs, state::Challenge, utils::hash_solutions};
use solana_program::pubkey::Pubkey;
use solana_program_test::*;

use solana_sdk::{
    account::{AccountSharedData, ReadableAccount},
    signer::Signer,
    transaction::Transaction,
};
use utils::add_challenge_account;

use crate::utils::{
    dump_account, get_deserialized, hash_solution, program_test,
};

mod utils;

fn add_challenge_with_solutions(
    context: &mut ProgramTestContext,
    solutions: Vec<&str>,
    authority: Option<Pubkey>,
) -> AccountSharedData {
    let solutions = hash_solutions(&solutions);
    add_challenge_account(
        context,
        Challenge {
            authority: authority.unwrap_or_else(|| context.payer.pubkey()),
            admit_cost: 200,
            tries_per_admit: 1,
            redeem: Pubkey::new_unique(),
            solving: 0,
            solutions,
        },
    )
}

#[tokio::test]
async fn add_solutions_creator_pays_to_empty_solutions() {
    let mut context = program_test().start_with_context().await;
    let creator = context.payer.pubkey();
    let added_acc = add_challenge_with_solutions(&mut context, vec![], None);

    let (challenge_pda, _) =
        Challenge::shank_pda(&challenge_id(), &context.payer.pubkey());

    dump_account::<Challenge>(&mut context, &challenge_pda).await;

    let solutions = vec!["hello", "world"];
    let ix = ixs::add_solutions(context.payer.pubkey(), creator, solutions)
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
        .expect("Failed add solutions");

    let (acc, value) =
        get_deserialized::<Challenge>(&mut context, &challenge_pda).await;

    assert_matches!(
        value,
        Challenge {
            authority,
            admit_cost: 200,
            tries_per_admit: 1,
            redeem: _,
            solving: 0,
            solutions,
        } => {
            assert_eq!(&authority, &creator);
            assert_eq!(solutions[0], hash_solution("hello"));
            assert_eq!(solutions[1], hash_solution("world"));
            assert_eq!(acc.data.len(), Challenge::needed_size(&solutions));
            assert_eq!(acc.lamports, added_acc.lamports(), "does not transfer lamports");
        }
    );
}

#[tokio::test]
async fn add_solutions_creator_not_payer_to_empty_solutions() {
    let mut context = program_test().start_with_context().await;
    let creator = Pubkey::new_unique();

    add_challenge_with_solutions(&mut context, vec![], Some(creator));

    let solutions = vec!["hello", "world"];

    let ix = ixs::add_solutions(context.payer.pubkey(), creator, solutions)
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
        .expect("Failed add solutions");

    let (challenge_pda, _) =
        Challenge::shank_pda(&challenge_id(), &context.payer.pubkey());
    let (acc, value) =
        get_deserialized::<Challenge>(&mut context, &challenge_pda).await;

    eprintln!("Challenge: {:?}", value);

    assert_matches!(
        value,
        Challenge {
            authority,
            admit_cost: 200,
            tries_per_admit: 1,
            redeem: _,
            solving: 0,
            solutions,
        } => {
            assert_eq!(&authority, &creator);
            assert_eq!(solutions[0], hash_solution("hello"));
            assert_eq!(solutions[1], hash_solution("world"));
            assert_eq!(acc.data.len(), Challenge::needed_size(&solutions));
        }
    );
}
