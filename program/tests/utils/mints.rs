use assert_matches::assert_matches;
use challenge::state::{Challenger, Redeem};
use solana_program::{program_option::COption, pubkey::Pubkey};
use solana_program_test::ProgramTestContext;
use solana_sdk::account::Account;
use spl_token::state::Mint;

use super::get_unpacked;

#[allow(unused)]
pub async fn get_mint(
    context: &mut ProgramTestContext,
    pubkey: &Pubkey,
) -> (Account, Mint) {
    get_unpacked::<Mint>(context, pubkey).await
}

#[allow(unused)]
pub async fn verify_minted_when_redeeming(
    context: &mut ProgramTestContext,
    mint_address: Pubkey,
    expected_mint_supply: u64,
    redeem: &Redeem,
    challenger: &Challenger,
) {
    let (_, mint) = get_mint(context, &mint_address).await;
    assert_eq!(mint.supply, expected_mint_supply, "mint supply");

    let (_, challenger_ata_value) = get_unpacked::<spl_token::state::Account>(
        context,
        &redeem.ata(&challenger.authority),
    )
    .await;

    assert_matches!(
        challenger_ata_value,
        spl_token::state::Account {
            mint: m,
            owner: o,
            amount: 1,
            delegate: COption::None,
            state: spl_token::state::AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        } => {
            assert_eq!(m, mint_address);
            assert_eq!(o, challenger.authority);
        }
    );
}
