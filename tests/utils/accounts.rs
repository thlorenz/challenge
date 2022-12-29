use borsh::{BorshDeserialize, BorshSerialize};
use challenge::{
    challenge_id,
    state::{Challenge, HasPda, HasSize, Redeem},
    utils::hash_solutions,
};
use solana_program::{
    borsh::try_from_slice_unchecked, program_option::COption,
    program_pack::Pack, pubkey::Pubkey, rent::Rent,
};
use solana_program_test::ProgramTestContext;
use solana_sdk::{
    account::{Account, AccountSharedData},
    signer::Signer,
};
use spl_token::state::Mint;

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

#[allow(unused)]
pub async fn get_unpacked<T: Pack>(
    context: &mut ProgramTestContext,
    pubkey: &Pubkey,
) -> (Account, T) {
    let acc = get_account(context, pubkey).await;

    let value = T::unpack_unchecked(&acc.data).expect("Unable to deserialize");
    (acc, value)
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

#[allow(unused)]
pub async fn dump_packed_account<T: Pack + std::fmt::Debug>(
    context: &mut ProgramTestContext,
    pubkey: &Pubkey,
) {
    let (acc, value) = get_unpacked::<T>(context, pubkey).await;
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
pub fn add_pda_account<T: HasSize + HasPda + BorshSerialize>(
    context: &mut ProgramTestContext,
    value: &T,
) -> Account {
    let (address, _) = value.pda();
    let lamports = rent_exempt_lamports(value);
    let space = value.size();

    let mut account = AccountSharedData::new(lamports, space, &challenge_id());
    account.set_data(value.try_to_vec().unwrap());
    context.set_account(&address, &account);

    account.into()
}

#[allow(unused)]
pub fn add_pack_account<T: Pack>(
    context: &mut ProgramTestContext,
    address: &Pubkey,
    value: &T,
    owner: &Pubkey,
) -> Account {
    let space = T::get_packed_len();
    let lamports = Rent::default().minimum_balance(space);

    let mut account = AccountSharedData::new(lamports, space, owner);
    let mut dst = vec![0u8; space];
    T::pack_into_slice(value, dst.as_mut_slice());
    account.set_data(dst);
    context.set_account(address, &account);

    account.into()
}

#[allow(unused)]
pub fn add_mint_account(
    context: &mut ProgramTestContext,
    address: &Pubkey,
    mint: &Mint,
) -> Account {
    add_pack_account(context, address, mint, &spl_token::id())
}

#[allow(unused)]
pub fn add_mint_to_redeem(
    context: &mut ProgramTestContext,
    redeem: &Redeem,
) -> Account {
    let (mint_pda, _) = redeem.pda();
    let mint = Mint {
        mint_authority: COption::Some(redeem.challenge_pda),
        supply: 0,
        decimals: 0,
        is_initialized: true,
        freeze_authority: COption::None,
    };
    add_mint_account(context, &mint_pda, &mint)
}

#[allow(unused)] // it actually is in 02_add_solutions.rs
pub fn add_challenge_with_solutions(
    context: &mut ProgramTestContext,
    id: &str,
    solutions: Vec<&str>,
    authority: Option<Pubkey>,
) -> Account {
    let solutions = hash_solutions(&solutions);
    add_pda_account(
        context,
        &Challenge {
            authority: authority.unwrap_or_else(|| context.payer.pubkey()),
            id: id.to_string(),
            started: false,
            finished: false,
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
    add_pda_account(
        context,
        &Challenge {
            authority: authority.unwrap_or_else(|| context.payer.pubkey()),
            id: id.to_string(),
            started: true,
            finished: false,
            admit_cost: 200,
            tries_per_admit: 1,
            redeem: Pubkey::new_unique(),
            solving: 0,
            solutions,
        },
    )
}
