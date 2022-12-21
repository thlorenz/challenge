use borsh::{BorshDeserialize, BorshSerialize};
use challenge::{challenge_id, state::Challenge};
use solana_program::{
    borsh::try_from_slice_unchecked, pubkey::Pubkey, rent::Rent,
};
use solana_program_test::ProgramTestContext;
use solana_sdk::account::{Account, AccountSharedData};

#[allow(unused)] // it actually is in 01_create_challenge.rs
pub async fn get_deserialized<T: BorshDeserialize>(
    context: &mut ProgramTestContext,
    pubkey: &Pubkey,
) -> (Account, T) {
    let acc = context
        .banks_client
        .get_account(*pubkey)
        .await
        .expect("account not found")
        .expect("account empty");

    let value: T =
        try_from_slice_unchecked(&acc.data).expect("Unable to deserialize");
    (acc, value)
}

#[allow(unused)]
pub async fn dump_account<T: BorshDeserialize + std::fmt::Debug>(
    context: &mut ProgramTestContext,
    pubkey: &Pubkey,
) {
    let (acc, value) = get_deserialized::<T>(context, pubkey).await;
    eprintln!("{:#?}", value);
    eprintln!("{:#?}", acc);
}

pub fn rent_exempt_lamports(challenge: &Challenge) -> u64 {
    let rent = Rent::default();
    rent.minimum_balance(challenge.size())
}

#[allow(unused)] // it actually is in 02_add_solutions.rs
pub fn add_challenge_account(
    context: &mut ProgramTestContext,
    challenge: Challenge,
) -> AccountSharedData {
    let (address, _) =
        Challenge::shank_pda(&challenge_id(), &challenge.authority);

    let lamports = rent_exempt_lamports(&challenge);
    let space = challenge.size();

    let mut account = AccountSharedData::new(lamports, space, &challenge_id());
    account.set_data(challenge.try_to_vec().unwrap());
    context.set_account(&address, &account);

    account
}
