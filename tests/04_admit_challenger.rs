#![cfg(feature = "test-sbf")]

use assert_matches::assert_matches;

use challenge::{
    ixs::{self, AdmitChallengerIx},
    state::{Challenge, Challenger},
    utils::hash_solutions,
};

use solana_program::pubkey::Pubkey;
use solana_program_test::*;

use solana_sdk::{signer::Signer, transaction::Transaction};
#[allow(unused)]
use utils::dump_account;
use utils::{add_challenge_account, airdrop_rent};

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
    add_challenge_account(
        &mut context,
        Challenge {
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
