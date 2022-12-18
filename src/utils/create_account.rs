use solana_program::{
    account_info::AccountInfo, msg, program::invoke,
    program_error::ProgramError, pubkey::Pubkey, rent::Rent,
    system_instruction, sysvar::Sysvar,
};

use crate::error::ChallengeError;

// The below two methods create an account owned by the program, namely they initialize
// the Challenge PDA.
// The required rent is deducted from the payer's account.
#[inline(always)]
fn transfer_lamports<'a>(
    payer_info: &AccountInfo<'a>,
    to_account_info: &AccountInfo<'a>,
    lamports: u64,
) -> Result<(), ProgramError> {
    msg!("transfer_lamports()");
    if payer_info.lamports() < lamports {
        msg!("Payer has only {} lamports", payer_info.lamports());
        return Err(ChallengeError::InsufficientFunds.into());
    }
    invoke(
        &system_instruction::transfer(
            payer_info.key,
            to_account_info.key,
            lamports,
        ),
        &[payer_info.clone(), to_account_info.clone()],
    )
}

pub struct AllocateAndAssignAccountArgs<'a> {
    pub payer_info: &'a AccountInfo<'a>,
    pub account_info: &'a AccountInfo<'a>,
    pub owner: &'a Pubkey,
    pub size: usize,
}

#[inline(always)]
pub fn allocate_account_and_assign_owner(
    args: AllocateAndAssignAccountArgs,
) -> Result<(), ProgramError> {
    let rent = Rent::get()?;
    let AllocateAndAssignAccountArgs {
        payer_info,
        account_info,
        owner,
        size,
    } = args;

    let required_lamports = rent
        .minimum_balance(size)
        .max(1)
        .saturating_sub(account_info.lamports());

    // 1. Transfer the required rent to the account
    if required_lamports > 0 {
        transfer_lamports(payer_info, account_info, required_lamports)?;
    }

    // 2. Allocate the space to hold data we'll set during mint initialization
    //    At this point the account is still owned by the system program
    msg!("create_account() allocate space");
    // TODO(thlorenz): may need to invoke_signed with seeds here
    invoke(
        &system_instruction::allocate(
            account_info.key,
            size.try_into().unwrap(),
        ),
        &[account_info.clone()],
    )?;

    // 3. Assign the owner of the account so that it can sign on its behalf
    //    In our case we set the spl_token as owner so we can init the mint
    msg!("create_account() assign to owning program");
    invoke(
        &system_instruction::assign(account_info.key, owner),
        &[account_info.clone()],
    )?;

    Ok(())
}
