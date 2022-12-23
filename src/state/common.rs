use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, borsh::try_from_slice_unchecked,
    program_error::ProgramError, pubkey::Pubkey,
};

use crate::utils::{assert_account_is_funded_and_has_data, assert_keys_equal};

// -----------------
// StateFromAccount
// -----------------
pub trait TryStateFromAccountUnchecked<T: BorshDeserialize> {
    fn try_state_from_account_unchecked(&self) -> Result<T, ProgramError>;
}

impl<'a, T: BorshDeserialize> TryStateFromAccountUnchecked<T>
    for AccountInfo<'a>
{
    /// NOTE: Deserializes without checking that the entire slice has been consumed
    fn try_state_from_account_unchecked(&self) -> Result<T, ProgramError> {
        assert_account_is_funded_and_has_data(self)?;

        let account = {
            let account_data = self.try_borrow_data()?;
            try_from_slice_unchecked::<T>(&account_data)?
        };

        Ok(account)
    }
}

pub trait TryStateFromAccount<T: BorshDeserialize> {
    fn try_state_from_account(&self) -> Result<T, ProgramError>;
}

impl<'a, T: BorshDeserialize> TryStateFromAccount<T> for AccountInfo<'a> {
    fn try_state_from_account(&self) -> Result<T, ProgramError> {
        assert_account_is_funded_and_has_data(self)?;

        let account = {
            let account_data = self.try_borrow_data()?;
            BorshDeserialize::try_from_slice(&account_data)?
        };

        Ok(account)
    }
}

// -----------------
// StateFromPdaAccount
// -----------------
pub struct StateFromPdaAccountValue<T> {
    pub state: T,
    pub pda: Pubkey,
    pub bump: u8,
}

pub trait TryStateFromPdaAccountUnchecked<
    T: BorshDeserialize,
    F: FnOnce() -> (Pubkey, u8),
>: TryStateFromAccountUnchecked<T>
{
    fn try_state_from_pda_account_unchecked(
        &self,
        get_pda_and_bump: F,
    ) -> Result<StateFromPdaAccountValue<T>, ProgramError>;
}

impl<'a, T: BorshDeserialize, F: FnOnce() -> (Pubkey, u8)>
    TryStateFromPdaAccountUnchecked<T, F> for AccountInfo<'a>
{
    /// Deserializes a the account state from the given account data and verifies the following:
    /// - the account is funded and initialized (has data)
    /// - the account's address matches the PDA provided
    ///
    /// NOTE: Deserializes without checking that the entire slice has been consumed
    ///
    /// - [get_pda_and_bump] is used to derive the PDA and bump
    fn try_state_from_pda_account_unchecked(
        &self,
        get_pda_and_bump: F,
    ) -> Result<StateFromPdaAccountValue<T>, ProgramError> {
        let (pda, bump) = get_pda_and_bump();
        let state: T = self.try_state_from_account_unchecked()?;

        assert_keys_equal(self.key, &pda, || {
            format!(
                "The derrived PDA ({}) does not match the address of the provided PDA account ({})",
                pda, self.key
            )
        })?;

        Ok(StateFromPdaAccountValue { state, pda, bump })
    }
}

pub trait TryStateFromPdaAccount<
    T: BorshDeserialize,
    F: FnOnce() -> (Pubkey, u8),
>: TryStateFromAccount<T>
{
    fn try_state_from_pda_account(
        &self,
        get_pda_and_bump: F,
    ) -> Result<StateFromPdaAccountValue<T>, ProgramError>;
}

impl<'a, T: BorshDeserialize, F: FnOnce() -> (Pubkey, u8)>
    TryStateFromPdaAccount<T, F> for AccountInfo<'a>
{
    /// Deserializes a the account state from the given account data and verifies the following:
    /// - the account is funded and initialized (has data)
    /// - the account's address matches the PDA provided
    ///
    /// - [get_pda_and_bump] is used to derive the PDA and bump
    fn try_state_from_pda_account(
        &self,
        get_pda_and_bump: F,
    ) -> Result<StateFromPdaAccountValue<T>, ProgramError> {
        let (pda, bump) = get_pda_and_bump();
        let state: T = self.try_state_from_account()?;

        assert_keys_equal(self.key, &pda, || {
            format!(
                "The derrived PDA ({}) does not match the address of the provided PDA account ({})",
                pda, self.key
            )
        })?;

        Ok(StateFromPdaAccountValue { state, pda, bump })
    }
}
