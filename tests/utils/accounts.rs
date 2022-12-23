use borsh::{BorshDeserialize, BorshSerialize};
use challenge::{
    challenge_id,
    state::{Challenge, Challenger, HasSize},
    utils::hash_solutions,
};
use solana_program::{
    borsh::try_from_slice_unchecked, pubkey::Pubkey, rent::Rent,
};
use solana_program_test::ProgramTestContext;
use solana_sdk::{
    account::{Account, AccountSharedData},
    signer::Signer,
};

#[allow(unused)]
pub async fn get_account(
    context: &mut ProgramTestContext,
    pubkey: &Pubkey,
) -> Account {
    context
        .banks_client
        .get_account(*pubkey)
        .await
        .expect("get_account(): account not found")
        .expect("get_account(): account empty")
}

#[allow(unused)] // it actually is in 01_create_challenge.rs
pub async fn get_deserialized<T: BorshDeserialize>(
    context: &mut ProgramTestContext,
    pubkey: &Pubkey,
) -> (Account, T) {
    let acc = get_account(context, pubkey).await;
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

pub fn rent_exempt_lamports<T: HasSize>(sized_acc: &T) -> u64 {
    let rent = Rent::default();
    rent.minimum_balance(sized_acc.size())
}

#[allow(unused)]
pub async fn airdrop_rent(
    context: &mut ProgramTestContext,
    address: &Pubkey,
    space: usize,
) -> u64 {
    let rent = Rent::default();
    let lamports = rent.minimum_balance(space);
    let account = AccountSharedData::new(lamports, space, address);
    context.set_account(address, &account);
    lamports
}

#[allow(unused)]
pub fn add_pda_account<
    T: HasSize + BorshSerialize,
    F: FnOnce() -> (Pubkey, u8),
>(
    context: &mut ProgramTestContext,
    value: &T,
    get_pda_and_bump: F,
) -> Account {
    let (address, _) = get_pda_and_bump();
    let lamports = rent_exempt_lamports(value);
    let space = value.size();

    let mut account = AccountSharedData::new(lamports, space, &challenge_id());
    account.set_data(value.try_to_vec().unwrap());
    context.set_account(&address, &account);

    account.into()
}

#[allow(unused)] // it actually is in 02_add_solutions.rs
pub fn add_challenge_account(
    context: &mut ProgramTestContext,
    challenge: Challenge,
) -> Account {
    add_pda_account(context, &challenge, || {
        Challenge::shank_pda(
            &challenge_id(),
            &challenge.authority,
            &challenge.id,
        )
    })
}

#[allow(unused)] // it actually is in 02_add_solutions.rs
pub fn add_challenger_account(
    context: &mut ProgramTestContext,
    challenger: Challenger,
) -> Account {
    add_pda_account(context, &challenger, || {
        Challenger::shank_pda(
            &challenge_id(),
            &challenger.challenge_pda,
            &challenger.authority,
        )
    })
}

#[allow(unused)] // it actually is in 02_add_solutions.rs
pub fn add_challenge_with_solutions(
    context: &mut ProgramTestContext,
    id: &str,
    solutions: Vec<&str>,
    authority: Option<Pubkey>,
) -> Account {
    let solutions = hash_solutions(&solutions);
    add_challenge_account(
        context,
        Challenge {
            authority: authority.unwrap_or_else(|| context.payer.pubkey()),
            id: id.to_string(),
            started: false,
            admit_cost: 200,
            tries_per_admit: 1,
            redeem: Pubkey::new_unique(),
            solving: 0,
            solutions,
        },
    )
}

#[allow(unused)] // it actually is in 02_add_solutions.rs
pub fn add_started_challenge_with_solutions(
    context: &mut ProgramTestContext,
    id: &str,
    solutions: Vec<&str>,
    authority: Option<Pubkey>,
) -> Account {
    let solutions = hash_solutions(&solutions);
    add_challenge_account(
        context,
        Challenge {
            authority: authority.unwrap_or_else(|| context.payer.pubkey()),
            id: id.to_string(),
            started: true,
            admit_cost: 200,
            tries_per_admit: 1,
            redeem: Pubkey::new_unique(),
            solving: 0,
            solutions,
        },
    )
}
