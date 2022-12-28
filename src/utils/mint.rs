use solana_program::{
    account_info::AccountInfo, msg, program::invoke,
    program_error::ProgramError, program_pack::Pack,
};
use spl_token::instruction::initialize_mint2;

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
