use borsh::BorshDeserialize;
use solana_program::{borsh::try_from_slice_unchecked, pubkey::Pubkey};
use solana_program_test::ProgramTestContext;
use solana_sdk::account::Account;

pub async fn get_deserialized<T: BorshDeserialize>(
    context: &mut ProgramTestContext,
    pubkey: &Pubkey,
) -> (Account, T) {
    let data = context
        .banks_client
        .get_account(*pubkey)
        .await
        .expect("account not found")
        .expect("account empty");

    let value: T =
        try_from_slice_unchecked(&data.data).expect("Unable to deserialize");
    (data, value)
}
