use solana_program::{
    account_info::AccountInfo,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
};
use spl_associated_token_account::{
    get_associated_token_address_with_program_id,
    instruction::create_associated_token_account_idempotent,
};
use spl_token::instruction::{initialize_mint2, mint_to};

use super::{
    allocate_account_and_assign_owner, assert_keys_equal,
    AllocateAndAssignAccountArgs,
};

struct InitMintArgs<'a> {
    pub mint_info: &'a AccountInfo<'a>,
    pub mint_authority_info: &'a AccountInfo<'a>,
    pub spl_token_program_info: &'a AccountInfo<'a>,
}

fn initialize_mint(args: InitMintArgs) -> Result<(), ProgramError> {
    let InitMintArgs {
        mint_info,
        mint_authority_info,
        spl_token_program_info,
    } = args;
    let mint_pubkey = mint_info.key;
    let mint_authority = mint_authority_info.key;
    let freeze_authority = None;
    let decimals = 0;

    invoke(
        &initialize_mint2(
            spl_token_program_info.key,
            mint_pubkey,
            mint_authority,
            freeze_authority,
            decimals,
        )?,
        // 0. `[writable]` The mint to initialize.
        &[mint_info.clone()],
    )?;

    Ok(())
}

pub struct CreateMintArgs<'a, 'b> {
    pub payer_info: &'a AccountInfo<'a>,
    pub mint_info: &'a AccountInfo<'a>,
    pub mint_authority_info: &'a AccountInfo<'a>,
    pub spl_token_program_info: &'a AccountInfo<'a>,
    pub signer_seeds: &'b [&'b [u8]],
}

pub fn create_mint(args: CreateMintArgs) -> Result<(), ProgramError> {
    msg!("  create_mint_for()");

    let CreateMintArgs {
        payer_info,
        mint_info,
        mint_authority_info,
        spl_token_program_info,
        signer_seeds,
    } = args;

    assert_keys_equal(spl_token_program_info.key, &spl_token::id(), || {
        format!(
            "'{}' does not match the spl_token program id",
            spl_token_program_info.key
        )
    })?;

    allocate_account_and_assign_owner(AllocateAndAssignAccountArgs {
        payer_info,
        account_info: mint_info,
        owner: spl_token_program_info.key,
        size: spl_token::state::Mint::LEN,
        signer_seeds,
    })?;

    initialize_mint(InitMintArgs {
        mint_info,
        mint_authority_info,
        spl_token_program_info,
    })
}

pub struct MintTokenArgs<'a, 'b> {
    pub payer_info: &'a AccountInfo<'a>,
    pub recvr_info: &'a AccountInfo<'a>,
    pub recvr_ata_info: &'a AccountInfo<'a>,
    pub mint_info: &'a AccountInfo<'a>,
    pub mint_authority_info: &'a AccountInfo<'a>,
    pub spl_token_program_info: &'a AccountInfo<'a>,
    pub signer_seeds: &'b [&'b [u8]],
}

pub fn mint_token_to_recvr(args: MintTokenArgs) -> Result<(), ProgramError> {
    let MintTokenArgs {
        payer_info,
        recvr_info,
        recvr_ata_info,
        mint_info,
        mint_authority_info,
        spl_token_program_info,
        signer_seeds,
    } = args;

    let ata = get_associated_token_address_with_program_id(
        recvr_info.key,
        mint_info.key,
        spl_token_program_info.key,
    );
    assert_keys_equal(recvr_ata_info.key, &ata, || {
        format!(
            "The provided ATA ('{}') does not match ('{}')",
            recvr_info.key, ata
        )
    })?;

    msg!("mint_token_to_recvr() Creating ATA",);
    invoke(
        &create_associated_token_account_idempotent(
            payer_info.key, // payer
            recvr_info.key, // recvr
            mint_info.key,  // mint
            spl_token_program_info.key,
        ),
        // 0. `[writeable,signer]` Funding account (must be a system account)
        // 1. `[writeable]` Associated token account address to be created
        // 2. `[]` Wallet address for the new associated token account (same as funding)
        // 3. `[]` The token mint for the new associated token account
        &[
            payer_info.clone(),
            recvr_ata_info.clone(),
            recvr_info.clone(),
            mint_info.clone(),
        ],
    )?;

    msg!(
        "mint_token_to_recvr() Minting ({}) to ATA ({})",
        mint_info.key,
        recvr_ata_info.key,
    );
    invoke_signed(
        &mint_to(
            spl_token_program_info.key,
            mint_info.key,           // mint
            recvr_ata_info.key,      // account
            mint_authority_info.key, // owner (mint authority)
            &[mint_authority_info.key],
            1,
        )?,
        // 0. `[writable]` The mint.
        // 1. `[writable]` The account to mint tokens to.
        // 2. `[signer]`   The mint's minting authority.
        &[
            mint_info.clone(),
            recvr_ata_info.clone(),
            mint_authority_info.clone(),
        ],
        &[signer_seeds],
    )?;

    Ok(())
}
