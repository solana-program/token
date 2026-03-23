mod setup;

use {
    crate::setup::TOKEN_PROGRAM_ID,
    mollusk_svm::{result::Check, sysvar::Sysvars, Mollusk},
    pinocchio_token_interface::{
        native_mint,
        state::{
            account::Account as TokenAccount, account_state::AccountState, load_mut_unchecked,
            load_unchecked,
        },
    },
    solana_account::Account,
    solana_program_pack::Pack,
    solana_program_test::tokio,
    solana_pubkey::Pubkey,
    solana_rent::Rent,
};

fn create_token_account(
    mint: &Pubkey,
    owner: &Pubkey,
    is_native: bool,
    amount: u64,
    program_owner: &Pubkey,
) -> Account {
    let space = size_of::<TokenAccount>();
    let mut lamports = Rent::default().minimum_balance(space);

    let mut data: Vec<u8> = vec![0u8; space];
    let token = unsafe { load_mut_unchecked::<TokenAccount>(data.as_mut_slice()).unwrap() };
    token.set_account_state(AccountState::Initialized);
    token.mint = *mint.as_array();
    token.owner = *owner.as_array();
    token.set_amount(amount);
    token.set_native(is_native);

    if is_native {
        token.set_native_amount(lamports);
        lamports = lamports.saturating_add(amount);
    }

    Account {
        lamports,
        data,
        owner: *program_owner,
        executable: false,
        ..Default::default()
    }
}

/// Creates a Mollusk instance with the default feature set, excluding the
/// `bpf_account_data_direct_mapping` feature.
fn mollusk() -> Mollusk {
    let mut mollusk = Mollusk::default();
    mollusk.add_program(&TOKEN_PROGRAM_ID, "pinocchio_token_program");
    mollusk
}

#[tokio::test]
async fn sync_native() {
    let native_mint = Pubkey::new_from_array(native_mint::ID);
    let authority_key = Pubkey::new_unique();

    // native account
    //   - amount: 1_000_000_000
    //   - lamports: 2_000_000_000
    let source_account_key = Pubkey::new_unique();
    let mut source_account =
        create_token_account(&native_mint, &authority_key, true, 0, &TOKEN_PROGRAM_ID);
    source_account.lamports += 2_000_000_000;

    let instruction = spl_token_interface::instruction::sync_native_with_rent_sysvar(
        &TOKEN_PROGRAM_ID,
        &source_account_key,
    )
    .unwrap();

    // Executes the sync_native instruction.

    let result = mollusk().process_and_validate_instruction_chain(
        &[(&instruction, &[Check::success()])],
        &[
            (source_account_key, source_account),
            Sysvars::default().keyed_account_for_rent_sysvar(),
        ],
    );

    result.resulting_accounts.iter().for_each(|(key, account)| {
        if *key == source_account_key {
            let token_account = spl_token_interface::state::Account::unpack(&account.data).unwrap();
            assert_eq!(token_account.amount, 2_000_000_000);
        }
    });
}

#[test]
fn sync_native_with_rent_change() {
    let native_mint = Pubkey::new_from_array(native_mint::ID);
    let authority_key = Pubkey::new_unique();

    // native account
    //   - amount: 1_000_000_000
    //   - lamports: 2_000_000_000
    let source_account_key = Pubkey::new_unique();
    let lamports = 2_000_000_000;
    let source_account = create_token_account(
        &native_mint,
        &authority_key,
        true,
        lamports,
        &TOKEN_PROGRAM_ID,
    );

    let instruction =
        spl_token_interface::instruction::sync_native(&TOKEN_PROGRAM_ID, &source_account_key)
            .unwrap();

    // Executes the sync_native instruction.

    let mut rent = Rent::default();
    rent.lamports_per_byte_year *= 2;

    let space = size_of::<TokenAccount>();
    let new_rent_exempt_reserve = rent.minimum_balance(space);
    let old_rent_exempt_reserve = source_account.lamports - lamports;
    let rent_difference = new_rent_exempt_reserve - old_rent_exempt_reserve;

    let mut test_mollusk = mollusk();
    test_mollusk.sysvars.rent = rent;
    let result = test_mollusk.process_and_validate_instruction_chain(
        &[(&instruction, &[Check::success()])],
        &[(source_account_key, source_account)],
    );

    result.resulting_accounts.iter().for_each(|(key, account)| {
        if *key == source_account_key {
            let token_account = unsafe { load_unchecked::<TokenAccount>(&account.data).unwrap() };
            assert_eq!(token_account.amount(), lamports - rent_difference);
            assert_eq!(token_account.native_amount(), Some(new_rent_exempt_reserve));
        }
    });
}
