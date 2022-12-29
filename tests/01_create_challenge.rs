#![cfg(feature = "test-sbf")]

use crate::utils::{get_deserialized, get_unpacked, hash_solution};
use assert_matches::assert_matches;
use challenge::{
    challenge_id, ixs,
    state::{Challenge, HasPda, Redeem},
};
use solana_program::{program_option::COption, pubkey::Pubkey};
use solana_program_test::*;

use solana_sdk::{signer::Signer, transaction::Transaction};
use spl_token::state::Mint;

#[allow(unused)]
use crate::utils::{dump_account, dump_packed_account};
use crate::utils::{ixs_custom, program_test};

mod utils;

const ID: &str = "challenge-id";

async fn assert_mint_for_challenge(
    context: &mut ProgramTestContext,
    challenge_pda: Pubkey,
) {
    let (mint_pda, _) = Redeem::new(challenge_pda).pda();
    let (_, value) = get_unpacked::<Mint>(context, &mint_pda).await;

    assert_matches!(
      value,
      Mint {
        mint_authority: ma,
        supply: 0,
        decimals: 0,
        is_initialized: true,
        freeze_authority: COption::None,
      } => {
        assert_eq!(ma, COption::Some(challenge_pda));
      }
    )
}

#[tokio::test]
async fn create_challenge_without_solutions() {
    let mut context = program_test().start_with_context().await;
    let creator = context.payer.pubkey();

    let ix = ixs::create_challenge(
        creator,
        creator,
        ID.to_string(),
        1000,
        1,
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

    {
        let (challenge_pda, _) =
            Challenge::shank_pda(&challenge_id(), &creator, ID);
        let (acc, value) =
            get_deserialized::<Challenge>(&mut context, &challenge_pda).await;

        assert_matches!(
            value,
            Challenge {
                authority,
                id,
                started: false,
                finished: false,
                admit_cost: 1000,
                tries_per_admit: 1,
                redeem: r,
                solving: 0,
                solutions,
            } => {
                assert_eq!(&authority, &creator);
                assert_eq!(id, ID);
                assert_eq!(r, Redeem::new(challenge_pda).pda().0);
                assert!(solutions.is_empty());
                assert_eq!(acc.data.len(), Challenge::needed_size(&solutions, ID));
            }
        );
        assert_mint_for_challenge(&mut context, challenge_pda).await;
    }
}

#[tokio::test]
async fn create_challenge_with_two_solutions() {
    let mut context = program_test().start_with_context().await;
    let creator = context.payer.pubkey();

    let ix = ixs::create_challenge(
        creator,
        creator,
        ID.to_string(),
        1000,
        1,
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
            started: false,
            finished: false,
            admit_cost: 1000,
            tries_per_admit: 1,
            redeem: r,
            solving: 0,
            solutions,
        } => {
            assert_eq!(&authority, &creator);
            assert_eq!(id, ID);
            assert_eq!(r, Redeem::new(challenge_pda).pda().0);
            assert_eq!(solutions.len(), 2);
            assert_eq!(solutions[0], hash_solution("hello"));
            assert_eq!(solutions[1], hash_solution("world"));
            assert_eq!(acc.data.len(), Challenge::needed_size(&solutions, ID));
        }
    );
    assert_mint_for_challenge(&mut context, challenge_pda).await;
}

#[tokio::test]
async fn create_two_challenges_same_creator_different_id() {
    let mut context = program_test().start_with_context().await;
    let creator = context.payer.pubkey();

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
                started: false,
                finished: false,
                admit_cost: 1000,
                tries_per_admit: 1,
                redeem: r,
                solving: 0,
                solutions,
            } => {
                assert_eq!(&authority, &creator);
                assert_eq!(id, fst_id);
                assert_eq!(r, Redeem::new(challenge_pda).pda().0);
                assert_eq!(solutions.len(), 2);
                assert_eq!(solutions[0], hash_solution("hello"));
                assert_eq!(solutions[1], hash_solution("world"));
                assert_eq!(acc.data.len(), Challenge::needed_size(&solutions, &id));
            }
        );
        assert_mint_for_challenge(&mut context, challenge_pda).await;
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
                started: false,
                finished: false,
                admit_cost: 2000,
                tries_per_admit: 2,
                redeem: r,
                solving: 0,
                solutions,
            } => {
                assert_eq!(&authority, &creator);
                assert_eq!(id, snd_id);
                assert_eq!(r, Redeem::new(challenge_pda).pda().0);
                assert_eq!(solutions.len(), 2);
                assert_eq!(solutions[0], hash_solution("hola"));
                assert_eq!(solutions[1], hash_solution("mundo"));
                assert_eq!(acc.data.len(), Challenge::needed_size(&solutions, &id));
            }
        );
        assert_mint_for_challenge(&mut context, challenge_pda).await;
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

    let ix = ixs_custom::create_challenge_with_pda(
        creator,
        creator,
        ID.to_string(),
        1000,
        1,
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

    // Try to create second Challenge (this fails)
    {
        let ix = ixs::create_challenge(
            creator,
            creator,
            snd_id.to_string(),
            2000,
            2,
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
