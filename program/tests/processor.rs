//! Program state processor tests

use {
    mollusk_svm::{result::Check, Mollusk},
    serial_test::serial,
    solana_account::{create_account_for_test, Account as SolanaAccount, ReadableAccount},
    solana_account_info::{AccountInfo, IntoAccountInfo},
    solana_instruction::Instruction,
    solana_program_error::{ProgramError, ProgramResult},
    solana_program_option::COption,
    solana_program_pack::Pack,
    solana_pubkey::Pubkey,
    solana_rent::Rent,
    solana_sdk_ids::sysvar::rent,
    spl_token::{
        error::TokenError,
        instruction::{
            amount_to_ui_amount, approve, approve_checked, burn, burn_checked, close_account,
            freeze_account, get_account_data_size, initialize_account, initialize_account2,
            initialize_account3, initialize_immutable_owner, initialize_mint, initialize_mint2,
            initialize_multisig, initialize_multisig2, mint_to, mint_to_checked, revoke,
            set_authority, sync_native, thaw_account, transfer, transfer_checked,
            ui_amount_to_amount, AuthorityType, MAX_SIGNERS,
        },
        state::{Account, AccountState, Mint, Multisig},
    },
    std::collections::HashMap,
};

fn do_process_instruction(
    instruction: Instruction,
    mut accounts: Vec<&mut SolanaAccount>,
    checks: &[Check],
) -> ProgramResult {
    // Prepare accounts for mollusk.
    let instruction_accounts: Vec<(Pubkey, SolanaAccount)> = instruction
        .accounts
        .iter()
        .zip(&accounts)
        .map(|(account_meta, account)| (account_meta.pubkey, (*account).clone()))
        .collect();

    let mollusk = Mollusk::new(&spl_token::ID, "spl_token");
    let result =
        mollusk.process_and_validate_instruction(&instruction, &instruction_accounts, checks);

    // Update accounts after the instruction is processed.
    for (original, (_, updated)) in accounts
        .iter_mut()
        .zip(result.resulting_accounts.into_iter())
    {
        original.data = updated.data().to_vec();
        original.lamports = updated.lamports();
        original.owner = *updated.owner();
    }

    result
        .raw_result
        .map_err(|e| ProgramError::try_from(e).unwrap())
}

fn do_process_instruction_dups(
    instruction: Instruction,
    account_infos: Vec<AccountInfo>,
    checks: &[Check],
) -> ProgramResult {
    let mut cached_accounts = HashMap::new();
    let mut dedup_accounts = Vec::new();

    // Dedup accounts for mollusk.
    account_infos.iter().for_each(|account_info| {
        if !cached_accounts.contains_key(account_info.key) {
            let account = SolanaAccount {
                lamports: account_info.lamports(),
                data: account_info.try_borrow_data().unwrap().to_vec(),
                owner: *account_info.owner,
                executable: account_info.executable,
                rent_epoch: account_info.rent_epoch,
            };
            dedup_accounts.push((*account_info.key, account));
            cached_accounts.insert(account_info.key, account_info);
        }
    });

    let mollusk = Mollusk::new(&spl_token::ID, "spl_token");
    let result = mollusk.process_and_validate_instruction(&instruction, &dedup_accounts, checks);

    // Update accounts after the instruction is processed.
    result
        .resulting_accounts
        .into_iter()
        .for_each(|(pubkey, account)| {
            let account = account.clone();
            let account_info = cached_accounts.get(&pubkey).unwrap();
            if account.data.is_empty() {
                // When the account is closed, the tests expect the data to
                // be zeroed.
                account_info.try_borrow_mut_data().unwrap().fill(0);
            } else {
                account_info
                    .try_borrow_mut_data()
                    .unwrap()
                    .copy_from_slice(account.data());
            }
            **account_info.try_borrow_mut_lamports().unwrap() = account.lamports();
            account_info.assign(account.owner());
        });

    result
        .raw_result
        .map_err(|e| ProgramError::try_from(e).unwrap())
}

fn rent_sysvar() -> SolanaAccount {
    create_account_for_test(&Rent::default())
}

fn mint_minimum_balance() -> u64 {
    Rent::default().minimum_balance(Mint::get_packed_len())
}

fn account_minimum_balance() -> u64 {
    Rent::default().minimum_balance(Account::get_packed_len())
}

fn multisig_minimum_balance() -> u64 {
    Rent::default().minimum_balance(Multisig::get_packed_len())
}

#[test]
fn test_initialize_mint() {
    let program_id = spl_token::id();
    let owner_key = Pubkey::new_unique();
    let mint_key = Pubkey::new_unique();
    let mut mint_account = SolanaAccount::new(42, Mint::get_packed_len(), &program_id);
    let mint2_key = Pubkey::new_unique();
    let mut mint2_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let mut rent_sysvar = rent_sysvar();

    // mint is not rent exempt
    assert_eq!(
        Err(TokenError::NotRentExempt.into()),
        do_process_instruction(
            initialize_mint(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
            vec![&mut mint_account, &mut rent_sysvar],
            &[Check::err(TokenError::NotRentExempt.into())],
        )
    );

    mint_account.lamports = mint_minimum_balance();

    // create new mint
    do_process_instruction(
        initialize_mint(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
        vec![&mut mint_account, &mut rent_sysvar],
        &[Check::success()],
    )
    .unwrap();

    // create twice
    assert_eq!(
        Err(TokenError::AlreadyInUse.into()),
        do_process_instruction(
            initialize_mint(&program_id, &mint_key, &owner_key, None, 2,).unwrap(),
            vec![&mut mint_account, &mut rent_sysvar],
            &[Check::err(TokenError::AlreadyInUse.into())],
        )
    );

    // create another mint that can freeze
    do_process_instruction(
        initialize_mint(&program_id, &mint2_key, &owner_key, Some(&owner_key), 2).unwrap(),
        vec![&mut mint2_account, &mut rent_sysvar],
        &[
            Check::success(),
            // freeze authority is set
            Check::account(&mint2_key)
                .data_slice(46, &[1, 0, 0, 0])
                .build(),
            // freeze authority matches owner
            Check::account(&mint2_key)
                .data_slice(50, owner_key.as_ref())
                .build(),
        ],
    )
    .unwrap();
    let mint = Mint::unpack_unchecked(&mint2_account.data).unwrap();
    assert_eq!(mint.freeze_authority, COption::Some(owner_key));
}

#[test]
fn test_initialize_mint2() {
    let program_id = spl_token::id();
    let owner_key = Pubkey::new_unique();
    let mint_key = Pubkey::new_unique();
    let mut mint_account = SolanaAccount::new(42, Mint::get_packed_len(), &program_id);
    let mint2_key = Pubkey::new_unique();
    let mut mint2_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);

    // mint is not rent exempt
    assert_eq!(
        Err(TokenError::NotRentExempt.into()),
        do_process_instruction(
            initialize_mint2(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
            vec![&mut mint_account],
            &[Check::err(TokenError::NotRentExempt.into())],
        )
    );

    mint_account.lamports = mint_minimum_balance();

    // create new mint
    do_process_instruction(
        initialize_mint2(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
        vec![&mut mint_account],
        &[Check::success()],
    )
    .unwrap();

    // create twice
    assert_eq!(
        Err(TokenError::AlreadyInUse.into()),
        do_process_instruction(
            initialize_mint2(&program_id, &mint_key, &owner_key, None, 2,).unwrap(),
            vec![&mut mint_account],
            &[Check::err(TokenError::AlreadyInUse.into())],
        )
    );

    // create another mint that can freeze
    do_process_instruction(
        initialize_mint2(&program_id, &mint2_key, &owner_key, Some(&owner_key), 2).unwrap(),
        vec![&mut mint2_account],
        &[
            Check::success(),
            // freeze authority is set
            Check::account(&mint2_key)
                .data_slice(46, &[1, 0, 0, 0])
                .build(),
            // freeze authority matches owner
            Check::account(&mint2_key)
                .data_slice(50, owner_key.as_ref())
                .build(),
        ],
    )
    .unwrap();
    let mint = Mint::unpack_unchecked(&mint2_account.data).unwrap();
    assert_eq!(mint.freeze_authority, COption::Some(owner_key));
}

#[test]
fn test_initialize_mint_account() {
    let program_id = spl_token::id();
    let account_key = Pubkey::new_unique();
    let mut account_account = SolanaAccount::new(42, Account::get_packed_len(), &program_id);
    let owner_key = Pubkey::new_unique();
    let mut owner_account = SolanaAccount::default();
    let mint_key = Pubkey::new_unique();
    let mut mint_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let mut rent_sysvar = rent_sysvar();

    // account is not rent exempt
    assert_eq!(
        Err(TokenError::NotRentExempt.into()),
        do_process_instruction(
            initialize_account(&program_id, &account_key, &mint_key, &owner_key).unwrap(),
            vec![
                &mut account_account,
                &mut mint_account,
                &mut owner_account,
                &mut rent_sysvar
            ],
            &[Check::err(TokenError::NotRentExempt.into())],
        )
    );

    account_account.lamports = account_minimum_balance();

    // mint is not valid (not initialized)
    assert_eq!(
        Err(TokenError::InvalidMint.into()),
        do_process_instruction(
            initialize_account(&program_id, &account_key, &mint_key, &owner_key).unwrap(),
            vec![
                &mut account_account,
                &mut mint_account,
                &mut owner_account,
                &mut rent_sysvar
            ],
            &[Check::err(TokenError::InvalidMint.into())],
        )
    );

    // create mint
    do_process_instruction(
        initialize_mint(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
        vec![&mut mint_account, &mut rent_sysvar],
        &[Check::success()],
    )
    .unwrap();

    // mint not owned by program
    let not_program_id = Pubkey::new_unique();
    mint_account.owner = not_program_id;
    assert_eq!(
        Err(ProgramError::IncorrectProgramId),
        do_process_instruction(
            initialize_account(&program_id, &account_key, &mint_key, &owner_key).unwrap(),
            vec![
                &mut account_account,
                &mut mint_account,
                &mut owner_account,
                &mut rent_sysvar
            ],
            &[Check::err(ProgramError::IncorrectProgramId)],
        )
    );
    mint_account.owner = program_id;

    // create account
    do_process_instruction(
        initialize_account(&program_id, &account_key, &mint_key, &owner_key).unwrap(),
        vec![
            &mut account_account,
            &mut mint_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();

    // create twice
    assert_eq!(
        Err(TokenError::AlreadyInUse.into()),
        do_process_instruction(
            initialize_account(&program_id, &account_key, &mint_key, &owner_key).unwrap(),
            vec![
                &mut account_account,
                &mut mint_account,
                &mut owner_account,
                &mut rent_sysvar
            ],
            &[Check::err(TokenError::AlreadyInUse.into())],
        )
    );
}

#[test]
fn test_transfer_dups() {
    let program_id = spl_token::id();
    let account1_key = Pubkey::new_unique();
    let mut account1_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let mut account1_info: AccountInfo = (&account1_key, true, &mut account1_account).into();
    let account2_key = Pubkey::new_unique();
    let mut account2_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let mut account2_info: AccountInfo = (&account2_key, false, &mut account2_account).into();
    let account3_key = Pubkey::new_unique();
    let mut account3_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let account3_info: AccountInfo = (&account3_key, false, &mut account3_account).into();
    let account4_key = Pubkey::new_unique();
    let mut account4_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let account4_info: AccountInfo = (&account4_key, true, &mut account4_account).into();
    let multisig_key = Pubkey::new_unique();
    let mut multisig_account = SolanaAccount::new(
        multisig_minimum_balance(),
        Multisig::get_packed_len(),
        &program_id,
    );
    let multisig_info: AccountInfo = (&multisig_key, true, &mut multisig_account).into();
    let owner_key = Pubkey::new_unique();
    let mut owner_account = SolanaAccount::default();
    let owner_info: AccountInfo = (&owner_key, true, &mut owner_account).into();
    let mint_key = Pubkey::new_unique();
    let mut mint_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let mint_info: AccountInfo = (&mint_key, false, &mut mint_account).into();
    let rent_key = rent::id();
    let mut rent_sysvar = rent_sysvar();
    let rent_info: AccountInfo = (&rent_key, false, &mut rent_sysvar).into();

    // create mint
    do_process_instruction_dups(
        initialize_mint(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
        vec![mint_info.clone(), rent_info.clone()],
        &[Check::success()],
    )
    .unwrap();

    // create account
    do_process_instruction_dups(
        initialize_account(&program_id, &account1_key, &mint_key, &account1_key).unwrap(),
        vec![
            account1_info.clone(),
            mint_info.clone(),
            account1_info.clone(),
            rent_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // create another account
    do_process_instruction_dups(
        initialize_account(&program_id, &account2_key, &mint_key, &owner_key).unwrap(),
        vec![
            account2_info.clone(),
            mint_info.clone(),
            owner_info.clone(),
            rent_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // mint to account
    do_process_instruction_dups(
        mint_to(&program_id, &mint_key, &account1_key, &owner_key, &[], 1000).unwrap(),
        vec![mint_info.clone(), account1_info.clone(), owner_info.clone()],
        &[Check::success()],
    )
    .unwrap();

    // source-owner transfer
    do_process_instruction_dups(
        transfer(
            &program_id,
            &account1_key,
            &account2_key,
            &account1_key,
            &[],
            500,
        )
        .unwrap(),
        vec![
            account1_info.clone(),
            account2_info.clone(),
            account1_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // source-owner TransferChecked
    do_process_instruction_dups(
        transfer_checked(
            &program_id,
            &account1_key,
            &mint_key,
            &account2_key,
            &account1_key,
            &[],
            500,
            2,
        )
        .unwrap(),
        vec![
            account1_info.clone(),
            mint_info.clone(),
            account2_info.clone(),
            account1_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // source-delegate transfer
    let mut account = Account::unpack_unchecked(&account1_info.data.borrow()).unwrap();
    account.amount = 1000;
    account.delegated_amount = 1000;
    account.delegate = COption::Some(account1_key);
    account.owner = owner_key;
    Account::pack(account, &mut account1_info.data.borrow_mut()).unwrap();

    do_process_instruction_dups(
        transfer(
            &program_id,
            &account1_key,
            &account2_key,
            &account1_key,
            &[],
            500,
        )
        .unwrap(),
        vec![
            account1_info.clone(),
            account2_info.clone(),
            account1_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // source-delegate TransferChecked
    do_process_instruction_dups(
        transfer_checked(
            &program_id,
            &account1_key,
            &mint_key,
            &account2_key,
            &account1_key,
            &[],
            500,
            2,
        )
        .unwrap(),
        vec![
            account1_info.clone(),
            mint_info.clone(),
            account2_info.clone(),
            account1_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // test destination-owner transfer
    do_process_instruction_dups(
        initialize_account(&program_id, &account3_key, &mint_key, &account2_key).unwrap(),
        vec![
            account3_info.clone(),
            mint_info.clone(),
            account2_info.clone(),
            rent_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();
    do_process_instruction_dups(
        mint_to(&program_id, &mint_key, &account3_key, &owner_key, &[], 1000).unwrap(),
        vec![mint_info.clone(), account3_info.clone(), owner_info.clone()],
        &[Check::success()],
    )
    .unwrap();

    account1_info.is_signer = false;
    account2_info.is_signer = true;
    do_process_instruction_dups(
        transfer(
            &program_id,
            &account3_key,
            &account2_key,
            &account2_key,
            &[],
            500,
        )
        .unwrap(),
        vec![
            account3_info.clone(),
            account2_info.clone(),
            account2_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // destination-owner TransferChecked
    do_process_instruction_dups(
        transfer_checked(
            &program_id,
            &account3_key,
            &mint_key,
            &account2_key,
            &account2_key,
            &[],
            500,
            2,
        )
        .unwrap(),
        vec![
            account3_info.clone(),
            mint_info.clone(),
            account2_info.clone(),
            account2_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // test source-multisig signer
    do_process_instruction_dups(
        initialize_multisig(&program_id, &multisig_key, &[&account4_key], 1).unwrap(),
        vec![
            multisig_info.clone(),
            rent_info.clone(),
            account4_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();

    do_process_instruction_dups(
        initialize_account(&program_id, &account4_key, &mint_key, &multisig_key).unwrap(),
        vec![
            account4_info.clone(),
            mint_info.clone(),
            multisig_info.clone(),
            rent_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();

    do_process_instruction_dups(
        mint_to(&program_id, &mint_key, &account4_key, &owner_key, &[], 1000).unwrap(),
        vec![mint_info.clone(), account4_info.clone(), owner_info.clone()],
        &[Check::success()],
    )
    .unwrap();

    // source-multisig-signer transfer
    do_process_instruction_dups(
        transfer(
            &program_id,
            &account4_key,
            &account2_key,
            &multisig_key,
            &[&account4_key],
            500,
        )
        .unwrap(),
        vec![
            account4_info.clone(),
            account2_info.clone(),
            multisig_info.clone(),
            account4_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // source-multisig-signer TransferChecked
    do_process_instruction_dups(
        transfer_checked(
            &program_id,
            &account4_key,
            &mint_key,
            &account2_key,
            &multisig_key,
            &[&account4_key],
            500,
            2,
        )
        .unwrap(),
        vec![
            account4_info.clone(),
            mint_info.clone(),
            account2_info.clone(),
            multisig_info.clone(),
            account4_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();
}

#[test]
fn test_transfer() {
    let program_id = spl_token::id();
    let account_key = Pubkey::new_unique();
    let mut account_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let account2_key = Pubkey::new_unique();
    let mut account2_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let account3_key = Pubkey::new_unique();
    let mut account3_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let delegate_key = Pubkey::new_unique();
    let mut delegate_account = SolanaAccount::default();
    let mismatch_key = Pubkey::new_unique();
    let mut mismatch_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let owner_key = Pubkey::new_unique();
    let mut owner_account = SolanaAccount::default();
    let owner2_key = Pubkey::new_unique();
    let mut owner2_account = SolanaAccount::default();
    let mint_key = Pubkey::new_unique();
    let mut mint_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let mint2_key = Pubkey::new_unique();
    let mut rent_sysvar = rent_sysvar();

    // create mint
    do_process_instruction(
        initialize_mint(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
        vec![&mut mint_account, &mut rent_sysvar],
        &[Check::success()],
    )
    .unwrap();

    // create account
    do_process_instruction(
        initialize_account(&program_id, &account_key, &mint_key, &owner_key).unwrap(),
        vec![
            &mut account_account,
            &mut mint_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();

    // create another account
    do_process_instruction(
        initialize_account(&program_id, &account2_key, &mint_key, &owner_key).unwrap(),
        vec![
            &mut account2_account,
            &mut mint_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();

    // create another account
    do_process_instruction(
        initialize_account(&program_id, &account3_key, &mint_key, &owner_key).unwrap(),
        vec![
            &mut account3_account,
            &mut mint_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();

    // create mismatch account
    do_process_instruction(
        initialize_account(&program_id, &mismatch_key, &mint_key, &owner_key).unwrap(),
        vec![
            &mut mismatch_account,
            &mut mint_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();
    let mut account = Account::unpack_unchecked(&mismatch_account.data).unwrap();
    account.mint = mint2_key;
    Account::pack(account, &mut mismatch_account.data).unwrap();

    // mint to account
    do_process_instruction(
        mint_to(&program_id, &mint_key, &account_key, &owner_key, &[], 1000).unwrap(),
        vec![&mut mint_account, &mut account_account, &mut owner_account],
        &[Check::success()],
    )
    .unwrap();

    // missing signer
    let mut instruction = transfer(
        &program_id,
        &account_key,
        &account2_key,
        &owner_key,
        &[],
        1000,
    )
    .unwrap();
    instruction.accounts[2].is_signer = false;
    assert_eq!(
        Err(ProgramError::MissingRequiredSignature),
        do_process_instruction(
            instruction,
            vec![
                &mut account_account,
                &mut account2_account,
                &mut owner_account,
            ],
            &[Check::err(ProgramError::MissingRequiredSignature)],
        )
    );

    // mismatch mint
    assert_eq!(
        Err(TokenError::MintMismatch.into()),
        do_process_instruction(
            transfer(
                &program_id,
                &account_key,
                &mismatch_key,
                &owner_key,
                &[],
                1000
            )
            .unwrap(),
            vec![
                &mut account_account,
                &mut mismatch_account,
                &mut owner_account,
            ],
            &[Check::err(TokenError::MintMismatch.into())],
        )
    );

    // missing owner
    assert_eq!(
        Err(TokenError::OwnerMismatch.into()),
        do_process_instruction(
            transfer(
                &program_id,
                &account_key,
                &account2_key,
                &owner2_key,
                &[],
                1000
            )
            .unwrap(),
            vec![
                &mut account_account,
                &mut account2_account,
                &mut owner2_account,
            ],
            &[Check::err(TokenError::OwnerMismatch.into())],
        )
    );

    // account not owned by program
    let not_program_id = Pubkey::new_unique();
    account_account.owner = not_program_id;
    assert_eq!(
        Err(ProgramError::IncorrectProgramId),
        do_process_instruction(
            transfer(&program_id, &account_key, &account2_key, &owner_key, &[], 0,).unwrap(),
            vec![
                &mut account_account,
                &mut account2_account,
                &mut owner2_account,
            ],
            &[Check::err(ProgramError::IncorrectProgramId)],
        )
    );
    account_account.owner = program_id;

    // account 2 not owned by program
    let not_program_id = Pubkey::new_unique();
    account2_account.owner = not_program_id;
    assert_eq!(
        Err(ProgramError::IncorrectProgramId),
        do_process_instruction(
            transfer(&program_id, &account_key, &account2_key, &owner_key, &[], 0,).unwrap(),
            vec![
                &mut account_account,
                &mut account2_account,
                &mut owner2_account,
            ],
            &[Check::err(ProgramError::IncorrectProgramId)],
        )
    );
    account2_account.owner = program_id;

    // transfer
    do_process_instruction(
        transfer(
            &program_id,
            &account_key,
            &account2_key,
            &owner_key,
            &[],
            1000,
        )
        .unwrap(),
        vec![
            &mut account_account,
            &mut account2_account,
            &mut owner_account,
        ],
        &[Check::success()],
    )
    .unwrap();

    // insufficient funds
    assert_eq!(
        Err(TokenError::InsufficientFunds.into()),
        do_process_instruction(
            transfer(&program_id, &account_key, &account2_key, &owner_key, &[], 1).unwrap(),
            vec![
                &mut account_account,
                &mut account2_account,
                &mut owner_account,
            ],
            &[Check::err(TokenError::InsufficientFunds.into())],
        )
    );

    // transfer half back
    do_process_instruction(
        transfer(
            &program_id,
            &account2_key,
            &account_key,
            &owner_key,
            &[],
            500,
        )
        .unwrap(),
        vec![
            &mut account2_account,
            &mut account_account,
            &mut owner_account,
        ],
        &[Check::success()],
    )
    .unwrap();

    // incorrect decimals
    assert_eq!(
        Err(TokenError::MintDecimalsMismatch.into()),
        do_process_instruction(
            transfer_checked(
                &program_id,
                &account2_key,
                &mint_key,
                &account_key,
                &owner_key,
                &[],
                1,
                10 // <-- incorrect decimals
            )
            .unwrap(),
            vec![
                &mut account2_account,
                &mut mint_account,
                &mut account_account,
                &mut owner_account,
            ],
            &[Check::err(TokenError::MintDecimalsMismatch.into())],
        )
    );

    // incorrect mint
    assert_eq!(
        Err(TokenError::MintMismatch.into()),
        do_process_instruction(
            transfer_checked(
                &program_id,
                &account2_key,
                &account3_key, // <-- incorrect mint
                &account_key,
                &owner_key,
                &[],
                1,
                2
            )
            .unwrap(),
            vec![
                &mut account2_account,
                &mut account3_account, // <-- incorrect mint
                &mut account_account,
                &mut owner_account,
            ],
            &[Check::err(TokenError::MintMismatch.into())],
        )
    );
    // transfer rest with explicit decimals
    do_process_instruction(
        transfer_checked(
            &program_id,
            &account2_key,
            &mint_key,
            &account_key,
            &owner_key,
            &[],
            500,
            2,
        )
        .unwrap(),
        vec![
            &mut account2_account,
            &mut mint_account,
            &mut account_account,
            &mut owner_account,
        ],
        &[Check::success()],
    )
    .unwrap();

    // insufficient funds
    assert_eq!(
        Err(TokenError::InsufficientFunds.into()),
        do_process_instruction(
            transfer(&program_id, &account2_key, &account_key, &owner_key, &[], 1).unwrap(),
            vec![
                &mut account2_account,
                &mut account_account,
                &mut owner_account,
            ],
            &[Check::err(TokenError::InsufficientFunds.into())],
        )
    );

    // approve delegate
    do_process_instruction(
        approve(
            &program_id,
            &account_key,
            &delegate_key,
            &owner_key,
            &[],
            100,
        )
        .unwrap(),
        vec![
            &mut account_account,
            &mut delegate_account,
            &mut owner_account,
        ],
        &[Check::success()],
    )
    .unwrap();

    // not a delegate of source account
    assert_eq!(
        Err(TokenError::OwnerMismatch.into()),
        do_process_instruction(
            transfer(
                &program_id,
                &account_key,
                &account2_key,
                &owner2_key, // <-- incorrect owner or delegate
                &[],
                1,
            )
            .unwrap(),
            vec![
                &mut account_account,
                &mut account2_account,
                &mut owner2_account,
            ],
            &[Check::err(TokenError::OwnerMismatch.into())],
        )
    );

    // insufficient funds approved via delegate
    assert_eq!(
        Err(TokenError::InsufficientFunds.into()),
        do_process_instruction(
            transfer(
                &program_id,
                &account_key,
                &account2_key,
                &delegate_key,
                &[],
                101
            )
            .unwrap(),
            vec![
                &mut account_account,
                &mut account2_account,
                &mut delegate_account,
            ],
            &[Check::err(TokenError::InsufficientFunds.into())],
        )
    );

    // transfer via delegate
    do_process_instruction(
        transfer(
            &program_id,
            &account_key,
            &account2_key,
            &delegate_key,
            &[],
            100,
        )
        .unwrap(),
        vec![
            &mut account_account,
            &mut account2_account,
            &mut delegate_account,
        ],
        &[Check::success()],
    )
    .unwrap();

    // insufficient funds approved via delegate
    assert_eq!(
        Err(TokenError::OwnerMismatch.into()),
        do_process_instruction(
            transfer(
                &program_id,
                &account_key,
                &account2_key,
                &delegate_key,
                &[],
                1
            )
            .unwrap(),
            vec![
                &mut account_account,
                &mut account2_account,
                &mut delegate_account,
            ],
            &[Check::err(TokenError::OwnerMismatch.into())],
        )
    );

    // transfer rest
    do_process_instruction(
        transfer(
            &program_id,
            &account_key,
            &account2_key,
            &owner_key,
            &[],
            900,
        )
        .unwrap(),
        vec![
            &mut account_account,
            &mut account2_account,
            &mut owner_account,
        ],
        &[Check::success()],
    )
    .unwrap();

    // approve delegate
    do_process_instruction(
        approve(
            &program_id,
            &account_key,
            &delegate_key,
            &owner_key,
            &[],
            100,
        )
        .unwrap(),
        vec![
            &mut account_account,
            &mut delegate_account,
            &mut owner_account,
        ],
        &[Check::success()],
    )
    .unwrap();

    // insufficient funds in source account via delegate
    assert_eq!(
        Err(TokenError::InsufficientFunds.into()),
        do_process_instruction(
            transfer(
                &program_id,
                &account_key,
                &account2_key,
                &delegate_key,
                &[],
                100
            )
            .unwrap(),
            vec![
                &mut account_account,
                &mut account2_account,
                &mut delegate_account,
            ],
            &[Check::err(TokenError::InsufficientFunds.into())],
        )
    );
}

#[test]
fn test_self_transfer() {
    let program_id = spl_token::id();
    let account_key = Pubkey::new_unique();
    let mut account_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let account2_key = Pubkey::new_unique();
    let mut account2_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let account3_key = Pubkey::new_unique();
    let mut account3_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let delegate_key = Pubkey::new_unique();
    let mut delegate_account = SolanaAccount::default();
    let owner_key = Pubkey::new_unique();
    let mut owner_account = SolanaAccount::default();
    let owner2_key = Pubkey::new_unique();
    let mut owner2_account = SolanaAccount::default();
    let mint_key = Pubkey::new_unique();
    let mut mint_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let mut rent_sysvar = rent_sysvar();

    // create mint
    do_process_instruction(
        initialize_mint(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
        vec![&mut mint_account, &mut rent_sysvar],
        &[Check::success()],
    )
    .unwrap();

    // create account
    do_process_instruction(
        initialize_account(&program_id, &account_key, &mint_key, &owner_key).unwrap(),
        vec![
            &mut account_account,
            &mut mint_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();

    // create another account
    do_process_instruction(
        initialize_account(&program_id, &account2_key, &mint_key, &owner_key).unwrap(),
        vec![
            &mut account2_account,
            &mut mint_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();

    // create another account
    do_process_instruction(
        initialize_account(&program_id, &account3_key, &mint_key, &owner_key).unwrap(),
        vec![
            &mut account3_account,
            &mut mint_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();

    // mint to account
    do_process_instruction(
        mint_to(&program_id, &mint_key, &account_key, &owner_key, &[], 1000).unwrap(),
        vec![&mut mint_account, &mut account_account, &mut owner_account],
        &[Check::success()],
    )
    .unwrap();

    let account_info = (&account_key, false, &mut account_account).into_account_info();
    let account3_info = (&account3_key, false, &mut account3_account).into_account_info();
    let delegate_info = (&delegate_key, true, &mut delegate_account).into_account_info();
    let owner_info = (&owner_key, true, &mut owner_account).into_account_info();
    let owner2_info = (&owner2_key, true, &mut owner2_account).into_account_info();
    let mint_info = (&mint_key, false, &mut mint_account).into_account_info();

    // transfer
    let instruction = transfer(
        &program_id,
        account_info.key,
        account_info.key,
        owner_info.key,
        &[],
        1000,
    )
    .unwrap();
    assert_eq!(
        Ok(()),
        do_process_instruction_dups(
            instruction,
            vec![
                account_info.clone(),
                account_info.clone(),
                owner_info.clone(),
            ],
            &[
                Check::success(),
                Check::account(account_info.key)
                    .data_slice(64, &1000u64.to_le_bytes())
                    .build()
            ],
        )
    );
    // no balance change...
    let account = Account::unpack_unchecked(&account_info.try_borrow_data().unwrap()).unwrap();
    assert_eq!(account.amount, 1000);

    // transfer checked
    let instruction = transfer_checked(
        &program_id,
        account_info.key,
        mint_info.key,
        account_info.key,
        owner_info.key,
        &[],
        1000,
        2,
    )
    .unwrap();
    assert_eq!(
        Ok(()),
        do_process_instruction_dups(
            instruction,
            vec![
                account_info.clone(),
                mint_info.clone(),
                account_info.clone(),
                owner_info.clone(),
            ],
            &[
                Check::success(),
                Check::account(account_info.key)
                    .data_slice(64, &1000u64.to_le_bytes())
                    .build()
            ],
        )
    );
    // no balance change...
    let account = Account::unpack_unchecked(&account_info.try_borrow_data().unwrap()).unwrap();
    assert_eq!(account.amount, 1000);

    // missing signer
    let mut owner_no_sign_info = owner_info.clone();
    let mut instruction = transfer(
        &program_id,
        account_info.key,
        account_info.key,
        owner_no_sign_info.key,
        &[],
        1000,
    )
    .unwrap();
    instruction.accounts[2].is_signer = false;
    owner_no_sign_info.is_signer = false;
    assert_eq!(
        Err(ProgramError::MissingRequiredSignature),
        do_process_instruction_dups(
            instruction,
            vec![
                account_info.clone(),
                account_info.clone(),
                owner_no_sign_info.clone(),
            ],
            &[Check::err(ProgramError::MissingRequiredSignature)],
        )
    );

    // missing signer checked
    let mut instruction = transfer_checked(
        &program_id,
        account_info.key,
        mint_info.key,
        account_info.key,
        owner_no_sign_info.key,
        &[],
        1000,
        2,
    )
    .unwrap();
    instruction.accounts[3].is_signer = false;
    assert_eq!(
        Err(ProgramError::MissingRequiredSignature),
        do_process_instruction_dups(
            instruction,
            vec![
                account_info.clone(),
                mint_info.clone(),
                account_info.clone(),
                owner_no_sign_info,
            ],
            &[Check::err(ProgramError::MissingRequiredSignature)],
        )
    );

    // missing owner
    let instruction = transfer(
        &program_id,
        account_info.key,
        account_info.key,
        owner2_info.key,
        &[],
        1000,
    )
    .unwrap();
    assert_eq!(
        Err(TokenError::OwnerMismatch.into()),
        do_process_instruction_dups(
            instruction,
            vec![
                account_info.clone(),
                account_info.clone(),
                owner2_info.clone(),
            ],
            &[Check::err(TokenError::OwnerMismatch.into())],
        )
    );

    // missing owner checked
    let instruction = transfer_checked(
        &program_id,
        account_info.key,
        mint_info.key,
        account_info.key,
        owner2_info.key,
        &[],
        1000,
        2,
    )
    .unwrap();
    assert_eq!(
        Err(TokenError::OwnerMismatch.into()),
        do_process_instruction_dups(
            instruction,
            vec![
                account_info.clone(),
                mint_info.clone(),
                account_info.clone(),
                owner2_info.clone(),
            ],
            &[Check::err(TokenError::OwnerMismatch.into())],
        )
    );

    // insufficient funds
    let instruction = transfer(
        &program_id,
        account_info.key,
        account_info.key,
        owner_info.key,
        &[],
        1001,
    )
    .unwrap();
    assert_eq!(
        Err(TokenError::InsufficientFunds.into()),
        do_process_instruction_dups(
            instruction,
            vec![
                account_info.clone(),
                account_info.clone(),
                owner_info.clone(),
            ],
            &[Check::err(TokenError::InsufficientFunds.into())],
        )
    );

    // insufficient funds checked
    let instruction = transfer_checked(
        &program_id,
        account_info.key,
        mint_info.key,
        account_info.key,
        owner_info.key,
        &[],
        1001,
        2,
    )
    .unwrap();
    assert_eq!(
        Err(TokenError::InsufficientFunds.into()),
        do_process_instruction_dups(
            instruction,
            vec![
                account_info.clone(),
                mint_info.clone(),
                account_info.clone(),
                owner_info.clone(),
            ],
            &[Check::err(TokenError::InsufficientFunds.into())],
        )
    );

    // incorrect decimals
    let instruction = transfer_checked(
        &program_id,
        account_info.key,
        mint_info.key,
        account_info.key,
        owner_info.key,
        &[],
        1,
        10, // <-- incorrect decimals
    )
    .unwrap();
    assert_eq!(
        Err(TokenError::MintDecimalsMismatch.into()),
        do_process_instruction_dups(
            instruction,
            vec![
                account_info.clone(),
                mint_info.clone(),
                account_info.clone(),
                owner_info.clone(),
            ],
            &[Check::err(TokenError::MintDecimalsMismatch.into())],
        )
    );

    // incorrect mint
    let instruction = transfer_checked(
        &program_id,
        account_info.key,
        account3_info.key, // <-- incorrect mint
        account_info.key,
        owner_info.key,
        &[],
        1,
        2,
    )
    .unwrap();
    assert_eq!(
        Err(TokenError::MintMismatch.into()),
        do_process_instruction_dups(
            instruction,
            vec![
                account_info.clone(),
                account3_info.clone(), // <-- incorrect mint
                account_info.clone(),
                owner_info.clone(),
            ],
            &[Check::err(TokenError::MintMismatch.into())],
        )
    );

    // approve delegate
    let instruction = approve(
        &program_id,
        account_info.key,
        delegate_info.key,
        owner_info.key,
        &[],
        100,
    )
    .unwrap();
    do_process_instruction_dups(
        instruction,
        vec![
            account_info.clone(),
            delegate_info.clone(),
            owner_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // delegate transfer
    let instruction = transfer(
        &program_id,
        account_info.key,
        account_info.key,
        delegate_info.key,
        &[],
        100,
    )
    .unwrap();
    assert_eq!(
        Ok(()),
        do_process_instruction_dups(
            instruction,
            vec![
                account_info.clone(),
                account_info.clone(),
                delegate_info.clone(),
            ],
            &[
                Check::success(),
                Check::account(account_info.key)
                    .data_slice(64, &1000u64.to_le_bytes())
                    .build(),
                Check::account(&account_key)
                    .data_slice(121, &100u64.to_le_bytes())
                    .build(),
            ],
        )
    );
    // no balance change...
    let account = Account::unpack_unchecked(&account_info.try_borrow_data().unwrap()).unwrap();
    assert_eq!(account.amount, 1000);
    assert_eq!(account.delegated_amount, 100);

    // delegate transfer checked
    let instruction = transfer_checked(
        &program_id,
        account_info.key,
        mint_info.key,
        account_info.key,
        delegate_info.key,
        &[],
        100,
        2,
    )
    .unwrap();
    assert_eq!(
        Ok(()),
        do_process_instruction_dups(
            instruction,
            vec![
                account_info.clone(),
                mint_info.clone(),
                account_info.clone(),
                delegate_info.clone(),
            ],
            &[
                Check::success(),
                Check::account(account_info.key)
                    .data_slice(64, &1000u64.to_le_bytes())
                    .build(),
                Check::account(&account_key)
                    .data_slice(121, &100u64.to_le_bytes())
                    .build(),
            ],
        )
    );
    // no balance change...
    let account = Account::unpack_unchecked(&account_info.try_borrow_data().unwrap()).unwrap();
    assert_eq!(account.amount, 1000);
    assert_eq!(account.delegated_amount, 100);

    // delegate insufficient funds
    let instruction = transfer(
        &program_id,
        account_info.key,
        account_info.key,
        delegate_info.key,
        &[],
        101,
    )
    .unwrap();
    assert_eq!(
        Err(TokenError::InsufficientFunds.into()),
        do_process_instruction_dups(
            instruction,
            vec![
                account_info.clone(),
                account_info.clone(),
                delegate_info.clone(),
            ],
            &[Check::err(TokenError::InsufficientFunds.into())],
        )
    );

    // delegate insufficient funds checked
    let instruction = transfer_checked(
        &program_id,
        account_info.key,
        mint_info.key,
        account_info.key,
        delegate_info.key,
        &[],
        101,
        2,
    )
    .unwrap();
    assert_eq!(
        Err(TokenError::InsufficientFunds.into()),
        do_process_instruction_dups(
            instruction,
            vec![
                account_info.clone(),
                mint_info.clone(),
                account_info.clone(),
                delegate_info.clone(),
            ],
            &[Check::err(TokenError::InsufficientFunds.into())],
        )
    );

    // owner transfer with delegate assigned
    let instruction = transfer(
        &program_id,
        account_info.key,
        account_info.key,
        owner_info.key,
        &[],
        1000,
    )
    .unwrap();
    assert_eq!(
        Ok(()),
        do_process_instruction_dups(
            instruction,
            vec![
                account_info.clone(),
                account_info.clone(),
                owner_info.clone(),
            ],
            &[
                Check::success(),
                Check::account(account_info.key)
                    .data_slice(64, &1000u64.to_le_bytes())
                    .build(),
            ],
        )
    );
    // no balance change...
    let account = Account::unpack_unchecked(&account_info.try_borrow_data().unwrap()).unwrap();
    assert_eq!(account.amount, 1000);

    // owner transfer with delegate assigned checked
    let instruction = transfer_checked(
        &program_id,
        account_info.key,
        mint_info.key,
        account_info.key,
        owner_info.key,
        &[],
        1000,
        2,
    )
    .unwrap();
    assert_eq!(
        Ok(()),
        do_process_instruction_dups(
            instruction,
            vec![
                account_info.clone(),
                mint_info.clone(),
                account_info.clone(),
                owner_info.clone(),
            ],
            &[
                Check::success(),
                Check::account(account_info.key)
                    .data_slice(64, &1000u64.to_le_bytes())
                    .build(),
            ],
        )
    );
    // no balance change...
    let account = Account::unpack_unchecked(&account_info.try_borrow_data().unwrap()).unwrap();
    assert_eq!(account.amount, 1000);
}

#[test]
fn test_mintable_token_with_zero_supply() {
    let program_id = spl_token::id();
    let account_key = Pubkey::new_unique();
    let mut account_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let owner_key = Pubkey::new_unique();
    let mut owner_account = SolanaAccount::default();
    let mint_key = Pubkey::new_unique();
    let mut mint_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let mut rent_sysvar = rent_sysvar();

    // create mint-able token with zero supply
    let decimals = 2;
    let expected_mint = Mint {
        mint_authority: COption::Some(owner_key),
        supply: 0,
        decimals,
        is_initialized: true,
        freeze_authority: COption::None,
    };
    let mut mint_data = [0u8; Mint::LEN];
    expected_mint.pack_into_slice(&mut mint_data);
    do_process_instruction(
        initialize_mint(&program_id, &mint_key, &owner_key, None, decimals).unwrap(),
        vec![&mut mint_account, &mut rent_sysvar],
        &[
            Check::success(),
            Check::account(&mint_key).data(&mint_data).build(),
        ],
    )
    .unwrap();
    let mint = Mint::unpack_unchecked(&mint_account.data).unwrap();
    assert_eq!(mint, expected_mint);

    // create account
    do_process_instruction(
        initialize_account(&program_id, &account_key, &mint_key, &owner_key).unwrap(),
        vec![
            &mut account_account,
            &mut mint_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();

    // mint to
    do_process_instruction(
        mint_to(&program_id, &mint_key, &account_key, &owner_key, &[], 42).unwrap(),
        vec![&mut mint_account, &mut account_account, &mut owner_account],
        &[
            Check::success(),
            Check::account(&account_key)
                .data_slice(64, &42u64.to_le_bytes())
                .build(),
        ],
    )
    .unwrap();
    let _ = Mint::unpack(&mint_account.data).unwrap();
    let account = Account::unpack_unchecked(&account_account.data).unwrap();
    assert_eq!(account.amount, 42);

    // mint to 2, with incorrect decimals
    assert_eq!(
        Err(TokenError::MintDecimalsMismatch.into()),
        do_process_instruction(
            mint_to_checked(
                &program_id,
                &mint_key,
                &account_key,
                &owner_key,
                &[],
                42,
                decimals + 1
            )
            .unwrap(),
            vec![&mut mint_account, &mut account_account, &mut owner_account],
            &[
                Check::err(TokenError::MintDecimalsMismatch.into()),
                Check::account(&account_key)
                    .data_slice(64, &42u64.to_le_bytes())
                    .build(),
            ],
        )
    );

    let _ = Mint::unpack(&mint_account.data).unwrap();
    let account = Account::unpack_unchecked(&account_account.data).unwrap();
    assert_eq!(account.amount, 42);

    // mint to 2
    do_process_instruction(
        mint_to_checked(
            &program_id,
            &mint_key,
            &account_key,
            &owner_key,
            &[],
            42,
            decimals,
        )
        .unwrap(),
        vec![&mut mint_account, &mut account_account, &mut owner_account],
        &[
            Check::success(),
            Check::account(&account_key)
                .data_slice(64, &84u64.to_le_bytes())
                .build(),
        ],
    )
    .unwrap();
    let _ = Mint::unpack(&mint_account.data).unwrap();
    let account = Account::unpack_unchecked(&account_account.data).unwrap();
    assert_eq!(account.amount, 84);
}

#[test]
fn test_approve_dups() {
    let program_id = spl_token::id();
    let account1_key = Pubkey::new_unique();
    let mut account1_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let account1_info: AccountInfo = (&account1_key, true, &mut account1_account).into();
    let account2_key = Pubkey::new_unique();
    let mut account2_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let account2_info: AccountInfo = (&account2_key, false, &mut account2_account).into();
    let account3_key = Pubkey::new_unique();
    let mut account3_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let account3_info: AccountInfo = (&account3_key, true, &mut account3_account).into();
    let multisig_key = Pubkey::new_unique();
    let mut multisig_account = SolanaAccount::new(
        multisig_minimum_balance(),
        Multisig::get_packed_len(),
        &program_id,
    );
    let multisig_info: AccountInfo = (&multisig_key, true, &mut multisig_account).into();
    let owner_key = Pubkey::new_unique();
    let mut owner_account = SolanaAccount::default();
    let owner_info: AccountInfo = (&owner_key, true, &mut owner_account).into();
    let mint_key = Pubkey::new_unique();
    let mut mint_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let mint_info: AccountInfo = (&mint_key, false, &mut mint_account).into();
    let rent_key = rent::id();
    let mut rent_sysvar = rent_sysvar();
    let rent_info: AccountInfo = (&rent_key, false, &mut rent_sysvar).into();

    // create mint
    do_process_instruction_dups(
        initialize_mint(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
        vec![mint_info.clone(), rent_info.clone()],
        &[Check::success()],
    )
    .unwrap();

    // create account
    do_process_instruction_dups(
        initialize_account(&program_id, &account1_key, &mint_key, &account1_key).unwrap(),
        vec![
            account1_info.clone(),
            mint_info.clone(),
            account1_info.clone(),
            rent_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // create another account
    do_process_instruction_dups(
        initialize_account(&program_id, &account2_key, &mint_key, &owner_key).unwrap(),
        vec![
            account2_info.clone(),
            mint_info.clone(),
            owner_info.clone(),
            rent_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // mint to account
    do_process_instruction_dups(
        mint_to(&program_id, &mint_key, &account1_key, &owner_key, &[], 1000).unwrap(),
        vec![mint_info.clone(), account1_info.clone(), owner_info.clone()],
        &[Check::success()],
    )
    .unwrap();

    // source-owner approve
    do_process_instruction_dups(
        approve(
            &program_id,
            &account1_key,
            &account2_key,
            &account1_key,
            &[],
            500,
        )
        .unwrap(),
        vec![
            account1_info.clone(),
            account2_info.clone(),
            account1_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // source-owner approve_checked
    do_process_instruction_dups(
        approve_checked(
            &program_id,
            &account1_key,
            &mint_key,
            &account2_key,
            &account1_key,
            &[],
            500,
            2,
        )
        .unwrap(),
        vec![
            account1_info.clone(),
            mint_info.clone(),
            account2_info.clone(),
            account1_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // source-owner revoke
    do_process_instruction_dups(
        revoke(&program_id, &account1_key, &account1_key, &[]).unwrap(),
        vec![account1_info.clone(), account1_info.clone()],
        &[Check::success()],
    )
    .unwrap();

    // test source-multisig signer
    do_process_instruction_dups(
        initialize_multisig(&program_id, &multisig_key, &[&account3_key], 1).unwrap(),
        vec![
            multisig_info.clone(),
            rent_info.clone(),
            account3_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();

    do_process_instruction_dups(
        initialize_account(&program_id, &account3_key, &mint_key, &multisig_key).unwrap(),
        vec![
            account3_info.clone(),
            mint_info.clone(),
            multisig_info.clone(),
            rent_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();

    do_process_instruction_dups(
        mint_to(&program_id, &mint_key, &account3_key, &owner_key, &[], 1000).unwrap(),
        vec![mint_info.clone(), account3_info.clone(), owner_info.clone()],
        &[Check::success()],
    )
    .unwrap();

    // source-multisig-signer approve
    do_process_instruction_dups(
        approve(
            &program_id,
            &account3_key,
            &account2_key,
            &multisig_key,
            &[&account3_key],
            500,
        )
        .unwrap(),
        vec![
            account3_info.clone(),
            account2_info.clone(),
            multisig_info.clone(),
            account3_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // source-multisig-signer approve_checked
    do_process_instruction_dups(
        approve_checked(
            &program_id,
            &account3_key,
            &mint_key,
            &account2_key,
            &multisig_key,
            &[&account3_key],
            500,
            2,
        )
        .unwrap(),
        vec![
            account3_info.clone(),
            mint_info.clone(),
            account2_info.clone(),
            multisig_info.clone(),
            account3_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // source-owner multisig-signer
    do_process_instruction_dups(
        revoke(&program_id, &account3_key, &multisig_key, &[&account3_key]).unwrap(),
        vec![
            account3_info.clone(),
            multisig_info.clone(),
            account3_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();
}

#[test]
fn test_approve() {
    let program_id = spl_token::id();
    let account_key = Pubkey::new_unique();
    let mut account_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let account2_key = Pubkey::new_unique();
    let mut account2_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let delegate_key = Pubkey::new_unique();
    let mut delegate_account = SolanaAccount::default();
    let owner_key = Pubkey::new_unique();
    let mut owner_account = SolanaAccount::default();
    let owner2_key = Pubkey::new_unique();
    let mut owner2_account = SolanaAccount::default();
    let mint_key = Pubkey::new_unique();
    let mut mint_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let mut rent_sysvar = rent_sysvar();

    // create mint
    do_process_instruction(
        initialize_mint(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
        vec![&mut mint_account, &mut rent_sysvar],
        &[Check::success()],
    )
    .unwrap();

    // create account
    do_process_instruction(
        initialize_account(&program_id, &account_key, &mint_key, &owner_key).unwrap(),
        vec![
            &mut account_account,
            &mut mint_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();

    // create another account
    do_process_instruction(
        initialize_account(&program_id, &account2_key, &mint_key, &owner_key).unwrap(),
        vec![
            &mut account2_account,
            &mut mint_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();

    // mint to account
    do_process_instruction(
        mint_to(&program_id, &mint_key, &account_key, &owner_key, &[], 1000).unwrap(),
        vec![&mut mint_account, &mut account_account, &mut owner_account],
        &[Check::success()],
    )
    .unwrap();

    // missing signer
    let mut instruction = approve(
        &program_id,
        &account_key,
        &delegate_key,
        &owner_key,
        &[],
        100,
    )
    .unwrap();
    instruction.accounts[2].is_signer = false;
    assert_eq!(
        Err(ProgramError::MissingRequiredSignature),
        do_process_instruction(
            instruction,
            vec![
                &mut account_account,
                &mut delegate_account,
                &mut owner_account,
            ],
            &[Check::err(ProgramError::MissingRequiredSignature)],
        )
    );

    // no owner
    assert_eq!(
        Err(TokenError::OwnerMismatch.into()),
        do_process_instruction(
            approve(
                &program_id,
                &account_key,
                &delegate_key,
                &owner2_key,
                &[],
                100
            )
            .unwrap(),
            vec![
                &mut account_account,
                &mut delegate_account,
                &mut owner2_account,
            ],
            &[Check::err(TokenError::OwnerMismatch.into())],
        )
    );

    // approve delegate
    do_process_instruction(
        approve(
            &program_id,
            &account_key,
            &delegate_key,
            &owner_key,
            &[],
            100,
        )
        .unwrap(),
        vec![
            &mut account_account,
            &mut delegate_account,
            &mut owner_account,
        ],
        &[Check::success()],
    )
    .unwrap();

    // approve delegate 2, with incorrect decimals
    assert_eq!(
        Err(TokenError::MintDecimalsMismatch.into()),
        do_process_instruction(
            approve_checked(
                &program_id,
                &account_key,
                &mint_key,
                &delegate_key,
                &owner_key,
                &[],
                100,
                0 // <-- incorrect decimals
            )
            .unwrap(),
            vec![
                &mut account_account,
                &mut mint_account,
                &mut delegate_account,
                &mut owner_account,
            ],
            &[Check::err(TokenError::MintDecimalsMismatch.into())],
        )
    );

    // approve delegate 2, with incorrect mint
    assert_eq!(
        Err(TokenError::MintMismatch.into()),
        do_process_instruction(
            approve_checked(
                &program_id,
                &account_key,
                &account2_key, // <-- bad mint
                &delegate_key,
                &owner_key,
                &[],
                100,
                0
            )
            .unwrap(),
            vec![
                &mut account_account,
                &mut account2_account, // <-- bad mint
                &mut delegate_account,
                &mut owner_account,
            ],
            &[Check::err(TokenError::MintMismatch.into())],
        )
    );

    // approve delegate 2
    do_process_instruction(
        approve_checked(
            &program_id,
            &account_key,
            &mint_key,
            &delegate_key,
            &owner_key,
            &[],
            100,
            2,
        )
        .unwrap(),
        vec![
            &mut account_account,
            &mut mint_account,
            &mut delegate_account,
            &mut owner_account,
        ],
        &[Check::success()],
    )
    .unwrap();

    // revoke delegate
    do_process_instruction(
        revoke(&program_id, &account_key, &owner_key, &[]).unwrap(),
        vec![&mut account_account, &mut owner_account],
        &[Check::success()],
    )
    .unwrap();
}

#[test]
fn test_set_authority_dups() {
    let program_id = spl_token::id();
    let account1_key = Pubkey::new_unique();
    let mut account1_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let account1_info: AccountInfo = (&account1_key, true, &mut account1_account).into();
    let owner_key = Pubkey::new_unique();
    let mint_key = Pubkey::new_unique();
    let mut mint_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let mint_info: AccountInfo = (&mint_key, true, &mut mint_account).into();
    let rent_key = rent::id();
    let mut rent_sysvar = rent_sysvar();
    let rent_info: AccountInfo = (&rent_key, false, &mut rent_sysvar).into();

    // create mint
    do_process_instruction_dups(
        initialize_mint(&program_id, &mint_key, &mint_key, Some(&mint_key), 2).unwrap(),
        vec![mint_info.clone(), rent_info.clone()],
        &[Check::success()],
    )
    .unwrap();

    // create account
    do_process_instruction_dups(
        initialize_account(&program_id, &account1_key, &mint_key, &account1_key).unwrap(),
        vec![
            account1_info.clone(),
            mint_info.clone(),
            account1_info.clone(),
            rent_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // set mint_authority when currently self
    do_process_instruction_dups(
        set_authority(
            &program_id,
            &mint_key,
            Some(&owner_key),
            AuthorityType::MintTokens,
            &mint_key,
            &[],
        )
        .unwrap(),
        vec![mint_info.clone(), mint_info.clone()],
        &[Check::success()],
    )
    .unwrap();

    // set freeze_authority when currently self
    do_process_instruction_dups(
        set_authority(
            &program_id,
            &mint_key,
            Some(&owner_key),
            AuthorityType::FreezeAccount,
            &mint_key,
            &[],
        )
        .unwrap(),
        vec![mint_info.clone(), mint_info.clone()],
        &[Check::success()],
    )
    .unwrap();

    // set account owner when currently self
    do_process_instruction_dups(
        set_authority(
            &program_id,
            &account1_key,
            Some(&owner_key),
            AuthorityType::AccountOwner,
            &account1_key,
            &[],
        )
        .unwrap(),
        vec![account1_info.clone(), account1_info.clone()],
        &[Check::success()],
    )
    .unwrap();

    // set close_authority when currently self
    let mut account = Account::unpack_unchecked(&account1_info.data.borrow()).unwrap();
    account.close_authority = COption::Some(account1_key);
    Account::pack(account, &mut account1_info.data.borrow_mut()).unwrap();

    do_process_instruction_dups(
        set_authority(
            &program_id,
            &account1_key,
            Some(&owner_key),
            AuthorityType::CloseAccount,
            &account1_key,
            &[],
        )
        .unwrap(),
        vec![account1_info.clone(), account1_info.clone()],
        &[Check::success()],
    )
    .unwrap();
}

#[test]
fn test_set_authority() {
    let program_id = spl_token::id();
    let account_key = Pubkey::new_unique();
    let mut account_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let account2_key = Pubkey::new_unique();
    let mut account2_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let owner_key = Pubkey::new_unique();
    let mut owner_account = SolanaAccount::default();
    let owner2_key = Pubkey::new_unique();
    let mut owner2_account = SolanaAccount::default();
    let owner3_key = Pubkey::new_unique();
    let mut owner3_account = SolanaAccount::default();
    let mint_key = Pubkey::new_unique();
    let mut mint_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let mint2_key = Pubkey::new_unique();
    let mut mint2_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let mut rent_sysvar = rent_sysvar();

    // create new mint with owner
    do_process_instruction(
        initialize_mint(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
        vec![&mut mint_account, &mut rent_sysvar],
        &[Check::success()],
    )
    .unwrap();

    // create mint with owner and freeze_authority
    do_process_instruction(
        initialize_mint(&program_id, &mint2_key, &owner_key, Some(&owner_key), 2).unwrap(),
        vec![&mut mint2_account, &mut rent_sysvar],
        &[Check::success()],
    )
    .unwrap();

    // invalid account
    assert_eq!(
        Err(ProgramError::UninitializedAccount),
        do_process_instruction(
            set_authority(
                &program_id,
                &account_key,
                Some(&owner2_key),
                AuthorityType::AccountOwner,
                &owner_key,
                &[]
            )
            .unwrap(),
            vec![&mut account_account, &mut owner_account],
            &[Check::err(ProgramError::UninitializedAccount)],
        )
    );

    // create account
    do_process_instruction(
        initialize_account(&program_id, &account_key, &mint_key, &owner_key).unwrap(),
        vec![
            &mut account_account,
            &mut mint_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();

    // create another account
    do_process_instruction(
        initialize_account(&program_id, &account2_key, &mint2_key, &owner_key).unwrap(),
        vec![
            &mut account2_account,
            &mut mint2_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();

    // missing owner
    assert_eq!(
        Err(TokenError::OwnerMismatch.into()),
        do_process_instruction(
            set_authority(
                &program_id,
                &account_key,
                Some(&owner_key),
                AuthorityType::AccountOwner,
                &owner2_key,
                &[]
            )
            .unwrap(),
            vec![&mut account_account, &mut owner2_account],
            &[Check::err(TokenError::OwnerMismatch.into())],
        )
    );

    // owner did not sign
    let mut instruction = set_authority(
        &program_id,
        &account_key,
        Some(&owner2_key),
        AuthorityType::AccountOwner,
        &owner_key,
        &[],
    )
    .unwrap();
    instruction.accounts[1].is_signer = false;
    assert_eq!(
        Err(ProgramError::MissingRequiredSignature),
        do_process_instruction(
            instruction,
            vec![&mut account_account, &mut owner_account,],
            &[Check::err(ProgramError::MissingRequiredSignature)]
        ),
    );

    // wrong authority type
    assert_eq!(
        Err(TokenError::AuthorityTypeNotSupported.into()),
        do_process_instruction(
            set_authority(
                &program_id,
                &account_key,
                Some(&owner2_key),
                AuthorityType::FreezeAccount,
                &owner_key,
                &[],
            )
            .unwrap(),
            vec![&mut account_account, &mut owner_account],
            &[Check::err(TokenError::AuthorityTypeNotSupported.into())],
        )
    );

    // account owner may not be set to None
    assert_eq!(
        Err(TokenError::InvalidInstruction.into()),
        do_process_instruction(
            set_authority(
                &program_id,
                &account_key,
                None,
                AuthorityType::AccountOwner,
                &owner_key,
                &[],
            )
            .unwrap(),
            vec![&mut account_account, &mut owner_account],
            &[Check::err(TokenError::InvalidInstruction.into())],
        )
    );

    // set delegate
    do_process_instruction(
        approve(
            &program_id,
            &account_key,
            &owner2_key,
            &owner_key,
            &[],
            u64::MAX,
        )
        .unwrap(),
        vec![
            &mut account_account,
            &mut owner2_account,
            &mut owner_account,
        ],
        &[
            Check::success(),
            // delegate set
            Check::account(&account_key)
                .data_slice(72, &[1, 0, 0, 0])
                .build(),
            // delegate
            Check::account(&account_key)
                .data_slice(76, owner2_key.as_ref())
                .build(),
            // delegated amount
            Check::account(&account_key)
                .data_slice(121, &u64::MAX.to_le_bytes())
                .build(),
        ],
    )
    .unwrap();
    let account = Account::unpack_unchecked(&account_account.data).unwrap();
    assert_eq!(account.delegate, COption::Some(owner2_key));
    assert_eq!(account.delegated_amount, u64::MAX);

    // set owner
    do_process_instruction(
        set_authority(
            &program_id,
            &account_key,
            Some(&owner3_key),
            AuthorityType::AccountOwner,
            &owner_key,
            &[],
        )
        .unwrap(),
        vec![&mut account_account, &mut owner_account],
        &[
            Check::success(),
            // delegate not set
            Check::account(&account_key)
                .data_slice(72, &[0, 0, 0, 0])
                .build(),
            // delegated amount
            Check::account(&account_key)
                .data_slice(121, &0u64.to_le_bytes())
                .build(),
        ],
    )
    .unwrap();

    // check delegate cleared
    let account = Account::unpack_unchecked(&account_account.data).unwrap();
    assert_eq!(account.delegate, COption::None);
    assert_eq!(account.delegated_amount, 0);

    // set owner without existing delegate
    do_process_instruction(
        set_authority(
            &program_id,
            &account_key,
            Some(&owner2_key),
            AuthorityType::AccountOwner,
            &owner3_key,
            &[],
        )
        .unwrap(),
        vec![&mut account_account, &mut owner3_account],
        &[Check::success()],
    )
    .unwrap();

    // set close_authority
    do_process_instruction(
        set_authority(
            &program_id,
            &account_key,
            Some(&owner2_key),
            AuthorityType::CloseAccount,
            &owner2_key,
            &[],
        )
        .unwrap(),
        vec![&mut account_account, &mut owner2_account],
        &[Check::success()],
    )
    .unwrap();

    // close_authority may be set to None
    do_process_instruction(
        set_authority(
            &program_id,
            &account_key,
            None,
            AuthorityType::CloseAccount,
            &owner2_key,
            &[],
        )
        .unwrap(),
        vec![&mut account_account, &mut owner2_account],
        &[Check::success()],
    )
    .unwrap();

    // wrong owner
    assert_eq!(
        Err(TokenError::OwnerMismatch.into()),
        do_process_instruction(
            set_authority(
                &program_id,
                &mint_key,
                Some(&owner3_key),
                AuthorityType::MintTokens,
                &owner2_key,
                &[]
            )
            .unwrap(),
            vec![&mut mint_account, &mut owner2_account],
            &[Check::err(TokenError::OwnerMismatch.into())],
        )
    );

    // owner did not sign
    let mut instruction = set_authority(
        &program_id,
        &mint_key,
        Some(&owner2_key),
        AuthorityType::MintTokens,
        &owner_key,
        &[],
    )
    .unwrap();
    instruction.accounts[1].is_signer = false;
    assert_eq!(
        Err(ProgramError::MissingRequiredSignature),
        do_process_instruction(
            instruction,
            vec![&mut mint_account, &mut owner_account],
            &[Check::err(ProgramError::MissingRequiredSignature)]
        ),
    );

    // cannot freeze
    assert_eq!(
        Err(TokenError::MintCannotFreeze.into()),
        do_process_instruction(
            set_authority(
                &program_id,
                &mint_key,
                Some(&owner2_key),
                AuthorityType::FreezeAccount,
                &owner_key,
                &[],
            )
            .unwrap(),
            vec![&mut mint_account, &mut owner_account],
            &[Check::err(TokenError::MintCannotFreeze.into())],
        )
    );

    // set owner
    do_process_instruction(
        set_authority(
            &program_id,
            &mint_key,
            Some(&owner2_key),
            AuthorityType::MintTokens,
            &owner_key,
            &[],
        )
        .unwrap(),
        vec![&mut mint_account, &mut owner_account],
        &[Check::success()],
    )
    .unwrap();

    // set owner to None
    do_process_instruction(
        set_authority(
            &program_id,
            &mint_key,
            None,
            AuthorityType::MintTokens,
            &owner2_key,
            &[],
        )
        .unwrap(),
        vec![&mut mint_account, &mut owner2_account],
        &[Check::success()],
    )
    .unwrap();

    // test unsetting mint_authority is one-way operation
    assert_eq!(
        Err(TokenError::FixedSupply.into()),
        do_process_instruction(
            set_authority(
                &program_id,
                &mint2_key,
                Some(&owner2_key),
                AuthorityType::MintTokens,
                &owner_key,
                &[]
            )
            .unwrap(),
            vec![&mut mint_account, &mut owner_account],
            &[Check::err(TokenError::FixedSupply.into())],
        )
    );

    // set freeze_authority
    do_process_instruction(
        set_authority(
            &program_id,
            &mint2_key,
            Some(&owner2_key),
            AuthorityType::FreezeAccount,
            &owner_key,
            &[],
        )
        .unwrap(),
        vec![&mut mint2_account, &mut owner_account],
        &[Check::success()],
    )
    .unwrap();

    // test unsetting freeze_authority is one-way operation
    do_process_instruction(
        set_authority(
            &program_id,
            &mint2_key,
            None,
            AuthorityType::FreezeAccount,
            &owner2_key,
            &[],
        )
        .unwrap(),
        vec![&mut mint2_account, &mut owner2_account],
        &[Check::success()],
    )
    .unwrap();

    assert_eq!(
        Err(TokenError::MintCannotFreeze.into()),
        do_process_instruction(
            set_authority(
                &program_id,
                &mint2_key,
                Some(&owner2_key),
                AuthorityType::FreezeAccount,
                &owner_key,
                &[],
            )
            .unwrap(),
            vec![&mut mint2_account, &mut owner2_account],
            &[Check::err(TokenError::MintCannotFreeze.into())],
        )
    );
}

#[test]
fn test_mint_to_dups() {
    let program_id = spl_token::id();
    let account1_key = Pubkey::new_unique();
    let mut account1_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let account1_info: AccountInfo = (&account1_key, true, &mut account1_account).into();
    let owner_key = Pubkey::new_unique();
    let mut owner_account = SolanaAccount::default();
    let owner_info: AccountInfo = (&owner_key, true, &mut owner_account).into();
    let mint_key = Pubkey::new_unique();
    let mut mint_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let mint_info: AccountInfo = (&mint_key, true, &mut mint_account).into();
    let rent_key = rent::id();
    let mut rent_sysvar = rent_sysvar();
    let rent_info: AccountInfo = (&rent_key, false, &mut rent_sysvar).into();

    // create mint
    do_process_instruction_dups(
        initialize_mint(&program_id, &mint_key, &mint_key, None, 2).unwrap(),
        vec![mint_info.clone(), rent_info.clone()],
        &[Check::success()],
    )
    .unwrap();

    // create account
    do_process_instruction_dups(
        initialize_account(&program_id, &account1_key, &mint_key, &owner_key).unwrap(),
        vec![
            account1_info.clone(),
            mint_info.clone(),
            owner_info.clone(),
            rent_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // mint_to when mint_authority is self
    do_process_instruction_dups(
        mint_to(&program_id, &mint_key, &account1_key, &mint_key, &[], 42).unwrap(),
        vec![mint_info.clone(), account1_info.clone(), mint_info.clone()],
        &[Check::success()],
    )
    .unwrap();

    // mint_to_checked when mint_authority is self
    do_process_instruction_dups(
        mint_to_checked(&program_id, &mint_key, &account1_key, &mint_key, &[], 42, 2).unwrap(),
        vec![mint_info.clone(), account1_info.clone(), mint_info.clone()],
        &[Check::success()],
    )
    .unwrap();

    // mint_to when mint_authority is account owner
    let mut mint = Mint::unpack_unchecked(&mint_info.data.borrow()).unwrap();
    mint.mint_authority = COption::Some(account1_key);
    Mint::pack(mint, &mut mint_info.data.borrow_mut()).unwrap();
    do_process_instruction_dups(
        mint_to(
            &program_id,
            &mint_key,
            &account1_key,
            &account1_key,
            &[],
            42,
        )
        .unwrap(),
        vec![
            mint_info.clone(),
            account1_info.clone(),
            account1_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // mint_to_checked when mint_authority is account owner
    do_process_instruction_dups(
        mint_to(
            &program_id,
            &mint_key,
            &account1_key,
            &account1_key,
            &[],
            42,
        )
        .unwrap(),
        vec![
            mint_info.clone(),
            account1_info.clone(),
            account1_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();
}

#[test]
fn test_mint_to() {
    let program_id = spl_token::id();
    let account_key = Pubkey::new_unique();
    let mut account_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let account2_key = Pubkey::new_unique();
    let mut account2_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let account3_key = Pubkey::new_unique();
    let mut account3_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let mismatch_key = Pubkey::new_unique();
    let mut mismatch_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let owner_key = Pubkey::new_unique();
    let mut owner_account = SolanaAccount::default();
    let owner2_key = Pubkey::new_unique();
    let mut owner2_account = SolanaAccount::default();
    let mint_key = Pubkey::new_unique();
    let mut mint_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let mint2_key = Pubkey::new_unique();
    let uninitialized_key = Pubkey::new_unique();
    let mut uninitialized_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let mut rent_sysvar = rent_sysvar();

    // create new mint with owner
    do_process_instruction(
        initialize_mint(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
        vec![&mut mint_account, &mut rent_sysvar],
        &[Check::success()],
    )
    .unwrap();

    // create account
    do_process_instruction(
        initialize_account(&program_id, &account_key, &mint_key, &owner_key).unwrap(),
        vec![
            &mut account_account,
            &mut mint_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();

    // create another account
    do_process_instruction(
        initialize_account(&program_id, &account2_key, &mint_key, &owner_key).unwrap(),
        vec![
            &mut account2_account,
            &mut mint_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();

    // create another account
    do_process_instruction(
        initialize_account(&program_id, &account3_key, &mint_key, &owner_key).unwrap(),
        vec![
            &mut account3_account,
            &mut mint_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();

    // create mismatch account
    do_process_instruction(
        initialize_account(&program_id, &mismatch_key, &mint_key, &owner_key).unwrap(),
        vec![
            &mut mismatch_account,
            &mut mint_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();
    let mut account = Account::unpack_unchecked(&mismatch_account.data).unwrap();
    account.mint = mint2_key;
    Account::pack(account, &mut mismatch_account.data).unwrap();

    // mint to
    do_process_instruction(
        mint_to(&program_id, &mint_key, &account_key, &owner_key, &[], 42).unwrap(),
        vec![&mut mint_account, &mut account_account, &mut owner_account],
        &[
            Check::success(),
            Check::account(&mint_key)
                .data_slice(36, &42u64.to_le_bytes())
                .build(),
            Check::account(&account_key)
                .data_slice(64, &42u64.to_le_bytes())
                .build(),
        ],
    )
    .unwrap();

    let mint = Mint::unpack_unchecked(&mint_account.data).unwrap();
    assert_eq!(mint.supply, 42);
    let account = Account::unpack_unchecked(&account_account.data).unwrap();
    assert_eq!(account.amount, 42);

    // mint to another account to test supply accumulation
    do_process_instruction(
        mint_to(&program_id, &mint_key, &account2_key, &owner_key, &[], 42).unwrap(),
        vec![&mut mint_account, &mut account2_account, &mut owner_account],
        &[
            Check::success(),
            Check::account(&mint_key)
                .data_slice(36, &84u64.to_le_bytes())
                .build(),
            Check::account(&account2_key)
                .data_slice(64, &42u64.to_le_bytes())
                .build(),
        ],
    )
    .unwrap();

    let mint = Mint::unpack_unchecked(&mint_account.data).unwrap();
    assert_eq!(mint.supply, 84);
    let account = Account::unpack_unchecked(&account2_account.data).unwrap();
    assert_eq!(account.amount, 42);

    // missing signer
    let mut instruction =
        mint_to(&program_id, &mint_key, &account2_key, &owner_key, &[], 42).unwrap();
    instruction.accounts[2].is_signer = false;
    assert_eq!(
        Err(ProgramError::MissingRequiredSignature),
        do_process_instruction(
            instruction,
            vec![&mut mint_account, &mut account2_account, &mut owner_account],
            &[Check::err(ProgramError::MissingRequiredSignature)],
        )
    );

    // mismatch account
    assert_eq!(
        Err(TokenError::MintMismatch.into()),
        do_process_instruction(
            mint_to(&program_id, &mint_key, &mismatch_key, &owner_key, &[], 42).unwrap(),
            vec![&mut mint_account, &mut mismatch_account, &mut owner_account],
            &[Check::err(TokenError::MintMismatch.into())],
        )
    );

    // missing owner
    assert_eq!(
        Err(TokenError::OwnerMismatch.into()),
        do_process_instruction(
            mint_to(&program_id, &mint_key, &account2_key, &owner2_key, &[], 42).unwrap(),
            vec![
                &mut mint_account,
                &mut account2_account,
                &mut owner2_account,
            ],
            &[Check::err(TokenError::OwnerMismatch.into())],
        )
    );

    // mint not owned by program
    let not_program_id = Pubkey::new_unique();
    mint_account.owner = not_program_id;
    assert_eq!(
        Err(ProgramError::IncorrectProgramId),
        do_process_instruction(
            mint_to(&program_id, &mint_key, &account_key, &owner_key, &[], 0).unwrap(),
            vec![&mut mint_account, &mut account_account, &mut owner_account],
            &[Check::err(ProgramError::IncorrectProgramId)],
        )
    );
    mint_account.owner = program_id;

    // account not owned by program
    let not_program_id = Pubkey::new_unique();
    account_account.owner = not_program_id;
    assert_eq!(
        Err(ProgramError::IncorrectProgramId),
        do_process_instruction(
            mint_to(&program_id, &mint_key, &account_key, &owner_key, &[], 0).unwrap(),
            vec![&mut mint_account, &mut account_account, &mut owner_account],
            &[Check::err(ProgramError::IncorrectProgramId)],
        )
    );
    account_account.owner = program_id;

    // uninitialized destination account
    assert_eq!(
        Err(ProgramError::UninitializedAccount),
        do_process_instruction(
            mint_to(
                &program_id,
                &mint_key,
                &uninitialized_key,
                &owner_key,
                &[],
                42
            )
            .unwrap(),
            vec![
                &mut mint_account,
                &mut uninitialized_account,
                &mut owner_account,
            ],
            &[Check::err(ProgramError::UninitializedAccount)],
        )
    );

    // unset mint_authority and test minting fails
    do_process_instruction(
        set_authority(
            &program_id,
            &mint_key,
            None,
            AuthorityType::MintTokens,
            &owner_key,
            &[],
        )
        .unwrap(),
        vec![&mut mint_account, &mut owner_account],
        &[Check::success()],
    )
    .unwrap();
    assert_eq!(
        Err(TokenError::FixedSupply.into()),
        do_process_instruction(
            mint_to(&program_id, &mint_key, &account2_key, &owner_key, &[], 42).unwrap(),
            vec![&mut mint_account, &mut account2_account, &mut owner_account],
            &[Check::err(TokenError::FixedSupply.into())],
        )
    );
}

#[test]
fn test_burn_dups() {
    let program_id = spl_token::id();
    let account1_key = Pubkey::new_unique();
    let mut account1_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let account1_info: AccountInfo = (&account1_key, true, &mut account1_account).into();
    let owner_key = Pubkey::new_unique();
    let mut owner_account = SolanaAccount::default();
    let owner_info: AccountInfo = (&owner_key, true, &mut owner_account).into();
    let mint_key = Pubkey::new_unique();
    let mut mint_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let mint_info: AccountInfo = (&mint_key, true, &mut mint_account).into();
    let rent_key = rent::id();
    let mut rent_sysvar = rent_sysvar();
    let rent_info: AccountInfo = (&rent_key, false, &mut rent_sysvar).into();

    // create mint
    do_process_instruction_dups(
        initialize_mint(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
        vec![mint_info.clone(), rent_info.clone()],
        &[Check::success()],
    )
    .unwrap();

    // create account
    do_process_instruction_dups(
        initialize_account(&program_id, &account1_key, &mint_key, &account1_key).unwrap(),
        vec![
            account1_info.clone(),
            mint_info.clone(),
            account1_info.clone(),
            rent_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // mint to account
    do_process_instruction_dups(
        mint_to(&program_id, &mint_key, &account1_key, &owner_key, &[], 1000).unwrap(),
        vec![mint_info.clone(), account1_info.clone(), owner_info.clone()],
        &[Check::success()],
    )
    .unwrap();

    // source-owner burn
    do_process_instruction_dups(
        burn(
            &program_id,
            &account1_key,
            &mint_key,
            &account1_key,
            &[],
            500,
        )
        .unwrap(),
        vec![
            account1_info.clone(),
            mint_info.clone(),
            account1_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // source-owner burn_checked
    do_process_instruction_dups(
        burn_checked(
            &program_id,
            &account1_key,
            &mint_key,
            &account1_key,
            &[],
            500,
            2,
        )
        .unwrap(),
        vec![
            account1_info.clone(),
            mint_info.clone(),
            account1_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // mint-owner burn
    do_process_instruction_dups(
        mint_to(&program_id, &mint_key, &account1_key, &owner_key, &[], 1000).unwrap(),
        vec![mint_info.clone(), account1_info.clone(), owner_info.clone()],
        &[Check::success()],
    )
    .unwrap();
    let mut account = Account::unpack_unchecked(&account1_info.data.borrow()).unwrap();
    account.owner = mint_key;
    Account::pack(account, &mut account1_info.data.borrow_mut()).unwrap();
    do_process_instruction_dups(
        burn(&program_id, &account1_key, &mint_key, &mint_key, &[], 500).unwrap(),
        vec![account1_info.clone(), mint_info.clone(), mint_info.clone()],
        &[Check::success()],
    )
    .unwrap();

    // mint-owner burn_checked
    do_process_instruction_dups(
        burn_checked(
            &program_id,
            &account1_key,
            &mint_key,
            &mint_key,
            &[],
            500,
            2,
        )
        .unwrap(),
        vec![account1_info.clone(), mint_info.clone(), mint_info.clone()],
        &[Check::success()],
    )
    .unwrap();

    // source-delegate burn
    do_process_instruction_dups(
        mint_to(&program_id, &mint_key, &account1_key, &owner_key, &[], 1000).unwrap(),
        vec![mint_info.clone(), account1_info.clone(), owner_info.clone()],
        &[Check::success()],
    )
    .unwrap();
    let mut account = Account::unpack_unchecked(&account1_info.data.borrow()).unwrap();
    account.delegated_amount = 1000;
    account.delegate = COption::Some(account1_key);
    account.owner = owner_key;
    Account::pack(account, &mut account1_info.data.borrow_mut()).unwrap();
    do_process_instruction_dups(
        burn(
            &program_id,
            &account1_key,
            &mint_key,
            &account1_key,
            &[],
            500,
        )
        .unwrap(),
        vec![
            account1_info.clone(),
            mint_info.clone(),
            account1_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // source-delegate burn_checked
    do_process_instruction_dups(
        burn_checked(
            &program_id,
            &account1_key,
            &mint_key,
            &account1_key,
            &[],
            500,
            2,
        )
        .unwrap(),
        vec![
            account1_info.clone(),
            mint_info.clone(),
            account1_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // mint-delegate burn
    do_process_instruction_dups(
        mint_to(&program_id, &mint_key, &account1_key, &owner_key, &[], 1000).unwrap(),
        vec![mint_info.clone(), account1_info.clone(), owner_info.clone()],
        &[Check::success()],
    )
    .unwrap();
    let mut account = Account::unpack_unchecked(&account1_info.data.borrow()).unwrap();
    account.delegated_amount = 1000;
    account.delegate = COption::Some(mint_key);
    account.owner = owner_key;
    Account::pack(account, &mut account1_info.data.borrow_mut()).unwrap();
    do_process_instruction_dups(
        burn(&program_id, &account1_key, &mint_key, &mint_key, &[], 500).unwrap(),
        vec![account1_info.clone(), mint_info.clone(), mint_info.clone()],
        &[Check::success()],
    )
    .unwrap();

    // mint-delegate burn_checked
    do_process_instruction_dups(
        burn_checked(
            &program_id,
            &account1_key,
            &mint_key,
            &mint_key,
            &[],
            500,
            2,
        )
        .unwrap(),
        vec![account1_info.clone(), mint_info.clone(), mint_info.clone()],
        &[Check::success()],
    )
    .unwrap();
}

#[test]
fn test_burn() {
    let program_id = spl_token::id();
    let account_key = Pubkey::new_unique();
    let mut account_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let account2_key = Pubkey::new_unique();
    let mut account2_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let account3_key = Pubkey::new_unique();
    let mut account3_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let delegate_key = Pubkey::new_unique();
    let mut delegate_account = SolanaAccount::default();
    let mismatch_key = Pubkey::new_unique();
    let mut mismatch_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let owner_key = Pubkey::new_unique();
    let mut owner_account = SolanaAccount::default();
    let owner2_key = Pubkey::new_unique();
    let mut owner2_account = SolanaAccount::default();
    let mint_key = Pubkey::new_unique();
    let mut mint_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let mint2_key = Pubkey::new_unique();
    let mut rent_sysvar = rent_sysvar();

    // create new mint
    do_process_instruction(
        initialize_mint(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
        vec![&mut mint_account, &mut rent_sysvar],
        &[Check::success()],
    )
    .unwrap();

    // create account
    do_process_instruction(
        initialize_account(&program_id, &account_key, &mint_key, &owner_key).unwrap(),
        vec![
            &mut account_account,
            &mut mint_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();

    // create another account
    do_process_instruction(
        initialize_account(&program_id, &account2_key, &mint_key, &owner_key).unwrap(),
        vec![
            &mut account2_account,
            &mut mint_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();

    // create another account
    do_process_instruction(
        initialize_account(&program_id, &account3_key, &mint_key, &owner_key).unwrap(),
        vec![
            &mut account3_account,
            &mut mint_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();

    // create mismatch account
    do_process_instruction(
        initialize_account(&program_id, &mismatch_key, &mint_key, &owner_key).unwrap(),
        vec![
            &mut mismatch_account,
            &mut mint_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();

    // mint to account
    do_process_instruction(
        mint_to(&program_id, &mint_key, &account_key, &owner_key, &[], 1000).unwrap(),
        vec![&mut mint_account, &mut account_account, &mut owner_account],
        &[Check::success()],
    )
    .unwrap();

    // mint to mismatch account and change mint key
    do_process_instruction(
        mint_to(&program_id, &mint_key, &mismatch_key, &owner_key, &[], 1000).unwrap(),
        vec![&mut mint_account, &mut mismatch_account, &mut owner_account],
        &[Check::success()],
    )
    .unwrap();
    let mut account = Account::unpack_unchecked(&mismatch_account.data).unwrap();
    account.mint = mint2_key;
    Account::pack(account, &mut mismatch_account.data).unwrap();

    // missing signer
    let mut instruction =
        burn(&program_id, &account_key, &mint_key, &delegate_key, &[], 42).unwrap();
    instruction.accounts[1].is_signer = false;
    assert_eq!(
        Err(TokenError::OwnerMismatch.into()),
        do_process_instruction(
            instruction,
            vec![
                &mut account_account,
                &mut mint_account,
                &mut delegate_account
            ],
            &[Check::err(TokenError::OwnerMismatch.into())],
        )
    );

    // missing owner
    assert_eq!(
        Err(TokenError::OwnerMismatch.into()),
        do_process_instruction(
            burn(&program_id, &account_key, &mint_key, &owner2_key, &[], 42).unwrap(),
            vec![&mut account_account, &mut mint_account, &mut owner2_account],
            &[Check::err(TokenError::OwnerMismatch.into())],
        )
    );

    // account not owned by program
    let not_program_id = Pubkey::new_unique();
    account_account.owner = not_program_id;
    assert_eq!(
        Err(ProgramError::IncorrectProgramId),
        do_process_instruction(
            burn(&program_id, &account_key, &mint_key, &owner_key, &[], 0).unwrap(),
            vec![&mut account_account, &mut mint_account, &mut owner_account],
            &[Check::err(ProgramError::IncorrectProgramId)],
        )
    );
    account_account.owner = program_id;

    // mint not owned by program
    let not_program_id = Pubkey::new_unique();
    mint_account.owner = not_program_id;
    assert_eq!(
        Err(ProgramError::IncorrectProgramId),
        do_process_instruction(
            burn(&program_id, &account_key, &mint_key, &owner_key, &[], 0).unwrap(),
            vec![&mut account_account, &mut mint_account, &mut owner_account],
            &[Check::err(ProgramError::IncorrectProgramId)],
        )
    );
    mint_account.owner = program_id;

    // mint mismatch
    assert_eq!(
        Err(TokenError::MintMismatch.into()),
        do_process_instruction(
            burn(&program_id, &mismatch_key, &mint_key, &owner_key, &[], 42).unwrap(),
            vec![&mut mismatch_account, &mut mint_account, &mut owner_account],
            &[Check::err(TokenError::MintMismatch.into())],
        )
    );

    // burn
    do_process_instruction(
        burn(&program_id, &account_key, &mint_key, &owner_key, &[], 21).unwrap(),
        vec![&mut account_account, &mut mint_account, &mut owner_account],
        &[Check::success()],
    )
    .unwrap();

    // burn_checked, with incorrect decimals
    assert_eq!(
        Err(TokenError::MintDecimalsMismatch.into()),
        do_process_instruction(
            burn_checked(&program_id, &account_key, &mint_key, &owner_key, &[], 21, 3).unwrap(),
            vec![&mut account_account, &mut mint_account, &mut owner_account],
            &[Check::err(TokenError::MintDecimalsMismatch.into())],
        )
    );

    // burn_checked
    do_process_instruction(
        burn_checked(&program_id, &account_key, &mint_key, &owner_key, &[], 21, 2).unwrap(),
        vec![&mut account_account, &mut mint_account, &mut owner_account],
        &[
            Check::success(),
            Check::account(&mint_key)
                .data_slice(36, &(2000u64 - 42).to_le_bytes())
                .build(),
            Check::account(&account_key)
                .data_slice(64, &(1000u64 - 42).to_le_bytes())
                .build(),
        ],
    )
    .unwrap();

    let mint = Mint::unpack_unchecked(&mint_account.data).unwrap();
    assert_eq!(mint.supply, 2000 - 42);
    let account = Account::unpack_unchecked(&account_account.data).unwrap();
    assert_eq!(account.amount, 1000 - 42);

    // insufficient funds
    assert_eq!(
        Err(TokenError::InsufficientFunds.into()),
        do_process_instruction(
            burn(
                &program_id,
                &account_key,
                &mint_key,
                &owner_key,
                &[],
                100_000_000
            )
            .unwrap(),
            vec![&mut account_account, &mut mint_account, &mut owner_account],
            &[Check::err(TokenError::InsufficientFunds.into())],
        )
    );

    // approve delegate
    do_process_instruction(
        approve(
            &program_id,
            &account_key,
            &delegate_key,
            &owner_key,
            &[],
            84,
        )
        .unwrap(),
        vec![
            &mut account_account,
            &mut delegate_account,
            &mut owner_account,
        ],
        &[Check::success()],
    )
    .unwrap();

    // not a delegate of source account
    assert_eq!(
        Err(TokenError::OwnerMismatch.into()),
        do_process_instruction(
            burn(
                &program_id,
                &account_key,
                &mint_key,
                &owner2_key, // <-- incorrect owner or delegate
                &[],
                1,
            )
            .unwrap(),
            vec![&mut account_account, &mut mint_account, &mut owner2_account],
            &[Check::err(TokenError::OwnerMismatch.into())],
        )
    );

    // insufficient funds approved via delegate
    assert_eq!(
        Err(TokenError::InsufficientFunds.into()),
        do_process_instruction(
            burn(&program_id, &account_key, &mint_key, &delegate_key, &[], 85).unwrap(),
            vec![
                &mut account_account,
                &mut mint_account,
                &mut delegate_account
            ],
            &[Check::err(TokenError::InsufficientFunds.into())],
        )
    );

    // burn via delegate
    do_process_instruction(
        burn(&program_id, &account_key, &mint_key, &delegate_key, &[], 84).unwrap(),
        vec![
            &mut account_account,
            &mut mint_account,
            &mut delegate_account,
        ],
        &[
            Check::success(),
            Check::account(&mint_key)
                .data_slice(36, &(2000u64 - 42 - 84).to_le_bytes())
                .build(),
            Check::account(&account_key)
                .data_slice(64, &(1000u64 - 42 - 84).to_le_bytes())
                .build(),
        ],
    )
    .unwrap();

    // match
    let mint = Mint::unpack_unchecked(&mint_account.data).unwrap();
    assert_eq!(mint.supply, 2000 - 42 - 84);
    let account = Account::unpack_unchecked(&account_account.data).unwrap();
    assert_eq!(account.amount, 1000 - 42 - 84);

    // insufficient funds approved via delegate
    assert_eq!(
        Err(TokenError::OwnerMismatch.into()),
        do_process_instruction(
            burn(&program_id, &account_key, &mint_key, &delegate_key, &[], 1).unwrap(),
            vec![
                &mut account_account,
                &mut mint_account,
                &mut delegate_account
            ],
            &[Check::err(TokenError::OwnerMismatch.into())],
        )
    );
}

#[test]
fn test_burn_and_close_system_and_incinerator_tokens() {
    let program_id = spl_token::id();
    let account_key = Pubkey::new_unique();
    let mut account_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let incinerator_account_key = Pubkey::new_unique();
    let mut incinerator_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let system_account_key = Pubkey::new_unique();
    let mut system_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let owner_key = Pubkey::new_unique();
    let mut owner_account = SolanaAccount::default();
    let recipient_key = Pubkey::new_unique();
    let mut recipient_account = SolanaAccount::default();
    let mut mock_incinerator_account = SolanaAccount::default();
    let mint_key = Pubkey::new_unique();
    let mut mint_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);

    // create new mint
    do_process_instruction(
        initialize_mint2(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
        vec![&mut mint_account],
        &[Check::success()],
    )
    .unwrap();

    // create account
    do_process_instruction(
        initialize_account3(&program_id, &account_key, &mint_key, &owner_key).unwrap(),
        vec![&mut account_account, &mut mint_account],
        &[Check::success()],
    )
    .unwrap();

    // create incinerator- and system-owned accounts
    do_process_instruction(
        initialize_account3(
            &program_id,
            &incinerator_account_key,
            &mint_key,
            &solana_sdk_ids::incinerator::id(),
        )
        .unwrap(),
        vec![&mut incinerator_account, &mut mint_account],
        &[Check::success()],
    )
    .unwrap();
    do_process_instruction(
        initialize_account3(
            &program_id,
            &system_account_key,
            &mint_key,
            &solana_sdk_ids::system_program::id(),
        )
        .unwrap(),
        vec![&mut system_account, &mut mint_account],
        &[Check::success()],
    )
    .unwrap();

    // mint to account
    do_process_instruction(
        mint_to(&program_id, &mint_key, &account_key, &owner_key, &[], 1000).unwrap(),
        vec![&mut mint_account, &mut account_account, &mut owner_account],
        &[Check::success()],
    )
    .unwrap();

    // transfer half to incinerator, half to system program
    do_process_instruction(
        transfer(
            &program_id,
            &account_key,
            &incinerator_account_key,
            &owner_key,
            &[],
            500,
        )
        .unwrap(),
        vec![
            &mut account_account,
            &mut incinerator_account,
            &mut owner_account,
        ],
        &[Check::success()],
    )
    .unwrap();
    do_process_instruction(
        transfer(
            &program_id,
            &account_key,
            &system_account_key,
            &owner_key,
            &[],
            500,
        )
        .unwrap(),
        vec![
            &mut account_account,
            &mut system_account,
            &mut owner_account,
        ],
        &[Check::success()],
    )
    .unwrap();

    // close with balance fails
    assert_eq!(
        Err(TokenError::NonNativeHasBalance.into()),
        do_process_instruction(
            close_account(
                &program_id,
                &incinerator_account_key,
                &solana_sdk_ids::incinerator::id(),
                &owner_key,
                &[]
            )
            .unwrap(),
            vec![
                &mut incinerator_account,
                &mut mock_incinerator_account,
                &mut owner_account,
            ],
            &[Check::err(TokenError::NonNativeHasBalance.into())],
        )
    );
    assert_eq!(
        Err(TokenError::NonNativeHasBalance.into()),
        do_process_instruction(
            close_account(
                &program_id,
                &system_account_key,
                &solana_sdk_ids::incinerator::id(),
                &owner_key,
                &[]
            )
            .unwrap(),
            vec![
                &mut system_account,
                &mut mock_incinerator_account,
                &mut owner_account,
            ],
            &[Check::err(TokenError::NonNativeHasBalance.into())],
        )
    );

    // anyone can burn
    do_process_instruction(
        burn(
            &program_id,
            &incinerator_account_key,
            &mint_key,
            &recipient_key,
            &[],
            500,
        )
        .unwrap(),
        vec![
            &mut incinerator_account,
            &mut mint_account,
            &mut recipient_account,
        ],
        &[Check::success()],
    )
    .unwrap();
    do_process_instruction(
        burn(
            &program_id,
            &system_account_key,
            &mint_key,
            &recipient_key,
            &[],
            500,
        )
        .unwrap(),
        vec![
            &mut system_account,
            &mut mint_account,
            &mut recipient_account,
        ],
        &[Check::success()],
    )
    .unwrap();

    // closing fails if destination is not the incinerator
    assert_eq!(
        Err(ProgramError::InvalidAccountData),
        do_process_instruction(
            close_account(
                &program_id,
                &incinerator_account_key,
                &recipient_key,
                &owner_key,
                &[]
            )
            .unwrap(),
            vec![
                &mut incinerator_account,
                &mut recipient_account,
                &mut owner_account,
            ],
            &[Check::err(ProgramError::InvalidAccountData)],
        )
    );
    assert_eq!(
        Err(ProgramError::InvalidAccountData),
        do_process_instruction(
            close_account(
                &program_id,
                &system_account_key,
                &recipient_key,
                &owner_key,
                &[]
            )
            .unwrap(),
            vec![
                &mut system_account,
                &mut recipient_account,
                &mut owner_account,
            ],
            &[Check::err(ProgramError::InvalidAccountData)],
        )
    );

    // closing succeeds with incinerator recipient
    do_process_instruction(
        close_account(
            &program_id,
            &incinerator_account_key,
            &solana_sdk_ids::incinerator::id(),
            &owner_key,
            &[],
        )
        .unwrap(),
        vec![
            &mut incinerator_account,
            &mut mock_incinerator_account,
            &mut owner_account,
        ],
        &[Check::success()],
    )
    .unwrap();

    do_process_instruction(
        close_account(
            &program_id,
            &system_account_key,
            &solana_sdk_ids::incinerator::id(),
            &owner_key,
            &[],
        )
        .unwrap(),
        vec![
            &mut system_account,
            &mut mock_incinerator_account,
            &mut owner_account,
        ],
        &[Check::success()],
    )
    .unwrap();
}

#[test]
fn test_multisig() {
    let program_id = spl_token::id();
    let mint_key = Pubkey::new_unique();
    let mut mint_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let account_key = Pubkey::new_unique();
    let mut account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let account2_key = Pubkey::new_unique();
    let mut account2_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let owner_key = Pubkey::new_unique();
    let mut owner_account = SolanaAccount::default();
    let multisig_key = Pubkey::new_unique();
    let mut multisig_account = SolanaAccount::new(42, Multisig::get_packed_len(), &program_id);
    let multisig_delegate_key = Pubkey::new_unique();
    let mut multisig_delegate_account = SolanaAccount::new(
        multisig_minimum_balance(),
        Multisig::get_packed_len(),
        &program_id,
    );
    let signer_keys = vec![Pubkey::new_unique(); MAX_SIGNERS];
    let signer_key_refs: Vec<&Pubkey> = signer_keys.iter().collect();
    let mut signer_accounts = vec![SolanaAccount::new(0, 0, &program_id); MAX_SIGNERS];
    let mut rent_sysvar = rent_sysvar();

    // multisig is not rent exempt
    let account_info_iter = &mut signer_accounts.iter_mut();
    assert_eq!(
        Err(TokenError::NotRentExempt.into()),
        do_process_instruction(
            initialize_multisig(&program_id, &multisig_key, &[&signer_keys[0]], 1).unwrap(),
            vec![
                &mut multisig_account,
                &mut rent_sysvar,
                account_info_iter.next().unwrap(),
            ],
            &[Check::err(TokenError::NotRentExempt.into())],
        )
    );

    multisig_account.lamports = multisig_minimum_balance();
    let mut multisig_account2 = multisig_account.clone();

    // single signer
    let account_info_iter = &mut signer_accounts.iter_mut();
    do_process_instruction(
        initialize_multisig(&program_id, &multisig_key, &[&signer_keys[0]], 1).unwrap(),
        vec![
            &mut multisig_account,
            &mut rent_sysvar,
            account_info_iter.next().unwrap(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // single signer using `initialize_multisig2`
    let account_info_iter = &mut signer_accounts.iter_mut();
    do_process_instruction(
        initialize_multisig2(&program_id, &multisig_key, &[&signer_keys[0]], 1).unwrap(),
        vec![&mut multisig_account2, account_info_iter.next().unwrap()],
        &[Check::success()],
    )
    .unwrap();

    // multiple signer
    let account_info_iter = &mut signer_accounts.iter_mut();
    do_process_instruction(
        initialize_multisig(
            &program_id,
            &multisig_delegate_key,
            &signer_key_refs,
            MAX_SIGNERS as u8,
        )
        .unwrap(),
        vec![
            &mut multisig_delegate_account,
            &mut rent_sysvar,
            account_info_iter.next().unwrap(),
            account_info_iter.next().unwrap(),
            account_info_iter.next().unwrap(),
            account_info_iter.next().unwrap(),
            account_info_iter.next().unwrap(),
            account_info_iter.next().unwrap(),
            account_info_iter.next().unwrap(),
            account_info_iter.next().unwrap(),
            account_info_iter.next().unwrap(),
            account_info_iter.next().unwrap(),
            account_info_iter.next().unwrap(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // create new mint with multisig owner
    do_process_instruction(
        initialize_mint(&program_id, &mint_key, &multisig_key, None, 2).unwrap(),
        vec![&mut mint_account, &mut rent_sysvar],
        &[Check::success()],
    )
    .unwrap();

    // create account with multisig owner
    do_process_instruction(
        initialize_account(&program_id, &account_key, &mint_key, &multisig_key).unwrap(),
        vec![
            &mut account,
            &mut mint_account,
            &mut multisig_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();

    // create another account with multisig owner
    do_process_instruction(
        initialize_account(
            &program_id,
            &account2_key,
            &mint_key,
            &multisig_delegate_key,
        )
        .unwrap(),
        vec![
            &mut account2_account,
            &mut mint_account,
            &mut multisig_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();

    // mint to account
    let account_info_iter = &mut signer_accounts.iter_mut();
    do_process_instruction(
        mint_to(
            &program_id,
            &mint_key,
            &account_key,
            &multisig_key,
            &[&signer_keys[0]],
            1000,
        )
        .unwrap(),
        vec![
            &mut mint_account,
            &mut account,
            &mut multisig_account,
            account_info_iter.next().unwrap(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // approve
    let account_info_iter = &mut signer_accounts.iter_mut();
    do_process_instruction(
        approve(
            &program_id,
            &account_key,
            &multisig_delegate_key,
            &multisig_key,
            &[&signer_keys[0]],
            100,
        )
        .unwrap(),
        vec![
            &mut account,
            &mut multisig_delegate_account,
            &mut multisig_account,
            account_info_iter.next().unwrap(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // transfer
    let account_info_iter = &mut signer_accounts.iter_mut();
    do_process_instruction(
        transfer(
            &program_id,
            &account_key,
            &account2_key,
            &multisig_key,
            &[&signer_keys[0]],
            42,
        )
        .unwrap(),
        vec![
            &mut account,
            &mut account2_account,
            &mut multisig_account,
            account_info_iter.next().unwrap(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // transfer via delegate
    let account_info_iter = &mut signer_accounts.iter_mut();
    do_process_instruction(
        transfer(
            &program_id,
            &account_key,
            &account2_key,
            &multisig_delegate_key,
            &signer_key_refs,
            42,
        )
        .unwrap(),
        vec![
            &mut account,
            &mut account2_account,
            &mut multisig_delegate_account,
            account_info_iter.next().unwrap(),
            account_info_iter.next().unwrap(),
            account_info_iter.next().unwrap(),
            account_info_iter.next().unwrap(),
            account_info_iter.next().unwrap(),
            account_info_iter.next().unwrap(),
            account_info_iter.next().unwrap(),
            account_info_iter.next().unwrap(),
            account_info_iter.next().unwrap(),
            account_info_iter.next().unwrap(),
            account_info_iter.next().unwrap(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // mint to
    let account_info_iter = &mut signer_accounts.iter_mut();
    do_process_instruction(
        mint_to(
            &program_id,
            &mint_key,
            &account2_key,
            &multisig_key,
            &[&signer_keys[0]],
            42,
        )
        .unwrap(),
        vec![
            &mut mint_account,
            &mut account2_account,
            &mut multisig_account,
            account_info_iter.next().unwrap(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // burn
    let account_info_iter = &mut signer_accounts.iter_mut();
    do_process_instruction(
        burn(
            &program_id,
            &account_key,
            &mint_key,
            &multisig_key,
            &[&signer_keys[0]],
            42,
        )
        .unwrap(),
        vec![
            &mut account,
            &mut mint_account,
            &mut multisig_account,
            account_info_iter.next().unwrap(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // burn via delegate
    let account_info_iter = &mut signer_accounts.iter_mut();
    do_process_instruction(
        burn(
            &program_id,
            &account_key,
            &mint_key,
            &multisig_delegate_key,
            &signer_key_refs,
            42,
        )
        .unwrap(),
        vec![
            &mut account,
            &mut mint_account,
            &mut multisig_delegate_account,
            account_info_iter.next().unwrap(),
            account_info_iter.next().unwrap(),
            account_info_iter.next().unwrap(),
            account_info_iter.next().unwrap(),
            account_info_iter.next().unwrap(),
            account_info_iter.next().unwrap(),
            account_info_iter.next().unwrap(),
            account_info_iter.next().unwrap(),
            account_info_iter.next().unwrap(),
            account_info_iter.next().unwrap(),
            account_info_iter.next().unwrap(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // freeze account
    let account3_key = Pubkey::new_unique();
    let mut account3_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let mint2_key = Pubkey::new_unique();
    let mut mint2_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    do_process_instruction(
        initialize_mint(
            &program_id,
            &mint2_key,
            &multisig_key,
            Some(&multisig_key),
            2,
        )
        .unwrap(),
        vec![&mut mint2_account, &mut rent_sysvar],
        &[Check::success()],
    )
    .unwrap();
    do_process_instruction(
        initialize_account(&program_id, &account3_key, &mint2_key, &owner_key).unwrap(),
        vec![
            &mut account3_account,
            &mut mint2_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();
    let account_info_iter = &mut signer_accounts.iter_mut();
    do_process_instruction(
        mint_to(
            &program_id,
            &mint2_key,
            &account3_key,
            &multisig_key,
            &[&signer_keys[0]],
            1000,
        )
        .unwrap(),
        vec![
            &mut mint2_account,
            &mut account3_account,
            &mut multisig_account,
            account_info_iter.next().unwrap(),
        ],
        &[Check::success()],
    )
    .unwrap();
    let account_info_iter = &mut signer_accounts.iter_mut();
    do_process_instruction(
        freeze_account(
            &program_id,
            &account3_key,
            &mint2_key,
            &multisig_key,
            &[&signer_keys[0]],
        )
        .unwrap(),
        vec![
            &mut account3_account,
            &mut mint2_account,
            &mut multisig_account,
            account_info_iter.next().unwrap(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // do SetAuthority on mint
    let account_info_iter = &mut signer_accounts.iter_mut();
    do_process_instruction(
        set_authority(
            &program_id,
            &mint_key,
            Some(&owner_key),
            AuthorityType::MintTokens,
            &multisig_key,
            &[&signer_keys[0]],
        )
        .unwrap(),
        vec![
            &mut mint_account,
            &mut multisig_account,
            account_info_iter.next().unwrap(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // do SetAuthority on account
    let account_info_iter = &mut signer_accounts.iter_mut();
    do_process_instruction(
        set_authority(
            &program_id,
            &account_key,
            Some(&owner_key),
            AuthorityType::AccountOwner,
            &multisig_key,
            &[&signer_keys[0]],
        )
        .unwrap(),
        vec![
            &mut account,
            &mut multisig_account,
            account_info_iter.next().unwrap(),
        ],
        &[Check::success()],
    )
    .unwrap();
}

#[test]
fn test_owner_close_account_dups() {
    let program_id = spl_token::id();
    let owner_key = Pubkey::new_unique();
    let mint_key = Pubkey::new_unique();
    let mut mint_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let mint_info: AccountInfo = (&mint_key, false, &mut mint_account).into();
    let rent_key = rent::id();
    let mut rent_sysvar = rent_sysvar();
    let rent_info: AccountInfo = (&rent_key, false, &mut rent_sysvar).into();

    // create mint
    do_process_instruction_dups(
        initialize_mint(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
        vec![mint_info.clone(), rent_info.clone()],
        &[Check::success()],
    )
    .unwrap();

    let to_close_key = Pubkey::new_unique();
    let mut to_close_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let to_close_account_info: AccountInfo = (&to_close_key, true, &mut to_close_account).into();
    let destination_account_key = Pubkey::new_unique();
    let mut destination_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let destination_account_info: AccountInfo =
        (&destination_account_key, true, &mut destination_account).into();
    // create account
    do_process_instruction_dups(
        initialize_account(&program_id, &to_close_key, &mint_key, &to_close_key).unwrap(),
        vec![
            to_close_account_info.clone(),
            mint_info.clone(),
            to_close_account_info.clone(),
            rent_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // source-owner close
    do_process_instruction_dups(
        close_account(
            &program_id,
            &to_close_key,
            &destination_account_key,
            &to_close_key,
            &[],
        )
        .unwrap(),
        vec![
            to_close_account_info.clone(),
            destination_account_info.clone(),
            to_close_account_info.clone(),
        ],
        &[
            Check::success(),
            Check::account(&to_close_key).data(&[]).build(),
        ],
    )
    .unwrap();
    assert_eq!(*to_close_account_info.data.borrow(), &[0u8; Account::LEN]);
}

#[test]
fn test_close_authority_close_account_dups() {
    let program_id = spl_token::id();
    let owner_key = Pubkey::new_unique();
    let mint_key = Pubkey::new_unique();
    let mut mint_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let mint_info: AccountInfo = (&mint_key, false, &mut mint_account).into();
    let rent_key = rent::id();
    let mut rent_sysvar = rent_sysvar();
    let rent_info: AccountInfo = (&rent_key, false, &mut rent_sysvar).into();

    // create mint
    do_process_instruction_dups(
        initialize_mint(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
        vec![mint_info.clone(), rent_info.clone()],
        &[Check::success()],
    )
    .unwrap();

    let to_close_key = Pubkey::new_unique();
    let mut to_close_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let to_close_account_info: AccountInfo = (&to_close_key, true, &mut to_close_account).into();
    let destination_account_key = Pubkey::new_unique();
    let mut destination_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let destination_account_info: AccountInfo =
        (&destination_account_key, true, &mut destination_account).into();
    // create account
    do_process_instruction_dups(
        initialize_account(&program_id, &to_close_key, &mint_key, &to_close_key).unwrap(),
        vec![
            to_close_account_info.clone(),
            mint_info.clone(),
            to_close_account_info.clone(),
            rent_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();
    let mut account = Account::unpack_unchecked(&to_close_account_info.data.borrow()).unwrap();
    account.close_authority = COption::Some(to_close_key);
    account.owner = owner_key;
    Account::pack(account, &mut to_close_account_info.data.borrow_mut()).unwrap();
    do_process_instruction_dups(
        close_account(
            &program_id,
            &to_close_key,
            &destination_account_key,
            &to_close_key,
            &[],
        )
        .unwrap(),
        vec![
            to_close_account_info.clone(),
            destination_account_info.clone(),
            to_close_account_info.clone(),
        ],
        &[
            Check::success(),
            Check::account(&to_close_key).data(&[]).build(),
        ],
    )
    .unwrap();
    assert_eq!(*to_close_account_info.data.borrow(), &[0u8; Account::LEN]);
}

#[test]
fn test_close_account() {
    let program_id = spl_token::id();
    let mint_key = Pubkey::new_unique();
    let mut mint_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let account_key = Pubkey::new_unique();
    let mut account_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let account2_key = Pubkey::new_unique();
    let mut account2_account = SolanaAccount::new(
        account_minimum_balance() + 42,
        Account::get_packed_len(),
        &program_id,
    );
    let account3_key = Pubkey::new_unique();
    let mut account3_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let owner_key = Pubkey::new_unique();
    let mut owner_account = SolanaAccount::default();
    let owner2_key = Pubkey::new_unique();
    let mut owner2_account = SolanaAccount::default();
    let mut rent_sysvar = rent_sysvar();

    // uninitialized
    assert_eq!(
        Err(ProgramError::UninitializedAccount),
        do_process_instruction(
            close_account(&program_id, &account_key, &account3_key, &owner2_key, &[]).unwrap(),
            vec![
                &mut account_account,
                &mut account3_account,
                &mut owner2_account,
            ],
            &[Check::err(ProgramError::UninitializedAccount)],
        )
    );

    // initialize and mint to non-native account
    do_process_instruction(
        initialize_mint(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
        vec![&mut mint_account, &mut rent_sysvar],
        &[Check::success()],
    )
    .unwrap();
    do_process_instruction(
        initialize_account(&program_id, &account_key, &mint_key, &owner_key).unwrap(),
        vec![
            &mut account_account,
            &mut mint_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();
    do_process_instruction(
        mint_to(&program_id, &mint_key, &account_key, &owner_key, &[], 42).unwrap(),
        vec![
            &mut mint_account,
            &mut account_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[
            Check::success(),
            Check::account(&account_key)
                .data_slice(64, &42u64.to_le_bytes())
                .build(),
        ],
    )
    .unwrap();
    let account = Account::unpack_unchecked(&account_account.data).unwrap();
    assert_eq!(account.amount, 42);

    // initialize native account
    do_process_instruction(
        initialize_account(
            &program_id,
            &account2_key,
            &spl_token::native_mint::id(),
            &owner_key,
        )
        .unwrap(),
        vec![
            &mut account2_account,
            &mut mint_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[
            Check::success(),
            Check::account(&account2_key)
                .data_slice(109, &[1, 0, 0, 0])
                .build(),
            Check::account(&account2_key)
                .data_slice(64, &42u64.to_le_bytes())
                .build(),
        ],
    )
    .unwrap();
    let account = Account::unpack_unchecked(&account2_account.data).unwrap();
    assert!(account.is_native());
    assert_eq!(account.amount, 42);

    // close non-native account with balance
    assert_eq!(
        Err(TokenError::NonNativeHasBalance.into()),
        do_process_instruction(
            close_account(&program_id, &account_key, &account3_key, &owner_key, &[]).unwrap(),
            vec![
                &mut account_account,
                &mut account3_account,
                &mut owner_account,
            ],
            &[
                Check::err(TokenError::NonNativeHasBalance.into()),
                Check::account(&account_key)
                    .lamports(account_minimum_balance())
                    .build()
            ],
        )
    );
    assert_eq!(account_account.lamports, account_minimum_balance());

    // empty account
    do_process_instruction(
        burn(&program_id, &account_key, &mint_key, &owner_key, &[], 42).unwrap(),
        vec![&mut account_account, &mut mint_account, &mut owner_account],
        &[Check::success()],
    )
    .unwrap();

    // wrong owner
    assert_eq!(
        Err(TokenError::OwnerMismatch.into()),
        do_process_instruction(
            close_account(&program_id, &account_key, &account3_key, &owner2_key, &[]).unwrap(),
            vec![
                &mut account_account,
                &mut account3_account,
                &mut owner2_account,
            ],
            &[Check::err(TokenError::OwnerMismatch.into())],
        )
    );

    // close account
    do_process_instruction(
        close_account(&program_id, &account_key, &account3_key, &owner_key, &[]).unwrap(),
        vec![
            &mut account_account,
            &mut account3_account,
            &mut owner_account,
        ],
        &[
            Check::success(),
            Check::account(&account_key).data(&[]).build(),
            Check::account(&account_key).lamports(0).build(),
            Check::account(&account3_key)
                .lamports(2 * account_minimum_balance())
                .build(),
        ],
    )
    .unwrap();
    assert!(account_account.data.is_empty());
    assert_eq!(account_account.lamports, 0);
    assert_eq!(account3_account.lamports, 2 * account_minimum_balance());

    // fund and initialize new non-native account to test close authority
    let account_key = Pubkey::new_unique();
    let mut account_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let owner2_key = Pubkey::new_unique();
    let mut owner2_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    do_process_instruction(
        initialize_account(&program_id, &account_key, &mint_key, &owner_key).unwrap(),
        vec![
            &mut account_account,
            &mut mint_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();
    account_account.lamports = 2;

    do_process_instruction(
        set_authority(
            &program_id,
            &account_key,
            Some(&owner2_key),
            AuthorityType::CloseAccount,
            &owner_key,
            &[],
        )
        .unwrap(),
        vec![&mut account_account, &mut owner_account],
        &[Check::success()],
    )
    .unwrap();

    // account owner cannot authorize close if close_authority is set
    assert_eq!(
        Err(TokenError::OwnerMismatch.into()),
        do_process_instruction(
            close_account(&program_id, &account_key, &account3_key, &owner_key, &[]).unwrap(),
            vec![
                &mut account_account,
                &mut account3_account,
                &mut owner_account,
            ],
            &[Check::err(TokenError::OwnerMismatch.into())],
        )
    );

    // close non-native account with close_authority
    do_process_instruction(
        close_account(&program_id, &account_key, &account3_key, &owner2_key, &[]).unwrap(),
        vec![
            &mut account_account,
            &mut account3_account,
            &mut owner2_account,
        ],
        &[
            Check::success(),
            Check::account(&account_key).data(&[]).build(),
            Check::account(&account_key).lamports(0).build(),
            Check::account(&account3_key)
                .lamports(2 * account_minimum_balance() + 2)
                .build(),
        ],
    )
    .unwrap();
    assert!(account_account.data.is_empty());
    assert_eq!(account_account.lamports, 0);
    assert_eq!(account3_account.lamports, 2 * account_minimum_balance() + 2);

    // close native account
    do_process_instruction(
        close_account(&program_id, &account2_key, &account3_key, &owner_key, &[]).unwrap(),
        vec![
            &mut account2_account,
            &mut account3_account,
            &mut owner_account,
        ],
        &[
            Check::success(),
            Check::account(&account2_key).data(&[]).build(),
            Check::account(&account3_key)
                .lamports(3 * account_minimum_balance() + 2 + 42)
                .build(),
        ],
    )
    .unwrap();
    assert!(account2_account.data.is_empty());
    assert_eq!(
        account3_account.lamports,
        3 * account_minimum_balance() + 2 + 42
    );
}

#[test]
fn test_native_token() {
    let program_id = spl_token::id();
    let mut mint_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let account_key = Pubkey::new_unique();
    let mut account_account = SolanaAccount::new(
        account_minimum_balance() + 40,
        Account::get_packed_len(),
        &program_id,
    );
    let account2_key = Pubkey::new_unique();
    let mut account2_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let account3_key = Pubkey::new_unique();
    let mut account3_account = SolanaAccount::new(account_minimum_balance(), 0, &program_id);
    let owner_key = Pubkey::new_unique();
    let mut owner_account = SolanaAccount::default();
    let owner2_key = Pubkey::new_unique();
    let mut owner2_account = SolanaAccount::default();
    let owner3_key = Pubkey::new_unique();
    let mut rent_sysvar = rent_sysvar();

    // initialize native account
    do_process_instruction(
        initialize_account(
            &program_id,
            &account_key,
            &spl_token::native_mint::id(),
            &owner_key,
        )
        .unwrap(),
        vec![
            &mut account_account,
            &mut mint_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[
            Check::success(),
            Check::account(&account_key)
                .data_slice(109, &[1, 0, 0, 0])
                .build(),
            Check::account(&account_key)
                .data_slice(64, &40u64.to_le_bytes())
                .build(),
        ],
    )
    .unwrap();
    let account = Account::unpack_unchecked(&account_account.data).unwrap();
    assert!(account.is_native());
    assert_eq!(account.amount, 40);

    // initialize native account
    do_process_instruction(
        initialize_account(
            &program_id,
            &account2_key,
            &spl_token::native_mint::id(),
            &owner_key,
        )
        .unwrap(),
        vec![
            &mut account2_account,
            &mut mint_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[
            Check::success(),
            Check::account(&account2_key)
                .data_slice(109, &[1, 0, 0, 0])
                .build(),
            Check::account(&account2_key)
                .data_slice(64, &0u64.to_le_bytes())
                .build(),
        ],
    )
    .unwrap();
    let account = Account::unpack_unchecked(&account2_account.data).unwrap();
    assert!(account.is_native());
    assert_eq!(account.amount, 0);

    // mint_to unsupported
    assert_eq!(
        Err(TokenError::NativeNotSupported.into()),
        do_process_instruction(
            mint_to(
                &program_id,
                &spl_token::native_mint::id(),
                &account_key,
                &owner_key,
                &[],
                42
            )
            .unwrap(),
            vec![&mut mint_account, &mut account_account, &mut owner_account],
            &[Check::err(TokenError::NativeNotSupported.into())],
        )
    );

    // burn unsupported
    let bogus_mint_key = Pubkey::new_unique();
    let mut bogus_mint_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    do_process_instruction(
        initialize_mint(&program_id, &bogus_mint_key, &owner_key, None, 2).unwrap(),
        vec![&mut bogus_mint_account, &mut rent_sysvar],
        &[Check::success()],
    )
    .unwrap();

    assert_eq!(
        Err(TokenError::NativeNotSupported.into()),
        do_process_instruction(
            burn(
                &program_id,
                &account_key,
                &bogus_mint_key,
                &owner_key,
                &[],
                42
            )
            .unwrap(),
            vec![
                &mut account_account,
                &mut bogus_mint_account,
                &mut owner_account
            ],
            &[Check::err(TokenError::NativeNotSupported.into())],
        )
    );

    // ensure can't transfer below rent-exempt reserve
    assert_eq!(
        Err(TokenError::InsufficientFunds.into()),
        do_process_instruction(
            transfer(
                &program_id,
                &account_key,
                &account2_key,
                &owner_key,
                &[],
                50,
            )
            .unwrap(),
            vec![
                &mut account_account,
                &mut account2_account,
                &mut owner_account,
            ],
            &[Check::err(TokenError::InsufficientFunds.into())],
        )
    );

    // transfer between native accounts
    do_process_instruction(
        transfer(
            &program_id,
            &account_key,
            &account2_key,
            &owner_key,
            &[],
            40,
        )
        .unwrap(),
        vec![
            &mut account_account,
            &mut account2_account,
            &mut owner_account,
        ],
        &[
            Check::success(),
            Check::account(&account_key)
                .lamports(account_minimum_balance())
                .build(),
            Check::account(&account_key)
                .data_slice(109, &[1, 0, 0, 0])
                .build(),
            Check::account(&account_key)
                .data_slice(64, &0u64.to_le_bytes())
                .build(),
            Check::account(&account2_key)
                .lamports(account_minimum_balance() + 40)
                .build(),
            Check::account(&account_key)
                .data_slice(109, &[1, 0, 0, 0])
                .build(),
            Check::account(&account2_key)
                .data_slice(64, &40u64.to_le_bytes())
                .build(),
        ],
    )
    .unwrap();
    assert_eq!(account_account.lamports, account_minimum_balance());
    let account = Account::unpack_unchecked(&account_account.data).unwrap();
    assert!(account.is_native());
    assert_eq!(account.amount, 0);
    assert_eq!(account2_account.lamports, account_minimum_balance() + 40);
    let account = Account::unpack_unchecked(&account2_account.data).unwrap();
    assert!(account.is_native());
    assert_eq!(account.amount, 40);

    // set close authority
    do_process_instruction(
        set_authority(
            &program_id,
            &account_key,
            Some(&owner3_key),
            AuthorityType::CloseAccount,
            &owner_key,
            &[],
        )
        .unwrap(),
        vec![&mut account_account, &mut owner_account],
        &[
            Check::success(),
            Check::account(&account_key)
                .data_slice(129, &[1, 0, 0, 0])
                .build(),
            Check::account(&account_key)
                .data_slice(133, owner3_key.as_ref())
                .build(),
        ],
    )
    .unwrap();
    let account = Account::unpack_unchecked(&account_account.data).unwrap();
    assert_eq!(account.close_authority, COption::Some(owner3_key));

    // set new account owner
    do_process_instruction(
        set_authority(
            &program_id,
            &account_key,
            Some(&owner2_key),
            AuthorityType::AccountOwner,
            &owner_key,
            &[],
        )
        .unwrap(),
        vec![&mut account_account, &mut owner_account],
        &[
            Check::success(),
            Check::account(&account_key)
                .data_slice(129, &[0, 0, 0, 0])
                .build(),
        ],
    )
    .unwrap();

    // close authority cleared
    let account = Account::unpack_unchecked(&account_account.data).unwrap();
    assert_eq!(account.close_authority, COption::None);

    // close native account
    do_process_instruction(
        close_account(&program_id, &account_key, &account3_key, &owner2_key, &[]).unwrap(),
        vec![
            &mut account_account,
            &mut account3_account,
            &mut owner2_account,
        ],
        &[
            Check::success(),
            Check::account(&account_key).lamports(0).build(),
            Check::account(&account3_key)
                .lamports(2 * account_minimum_balance())
                .build(),
            Check::account(&account_key).data(&[]).build(),
        ],
    )
    .unwrap();
    assert_eq!(account_account.lamports, 0);
    assert_eq!(account3_account.lamports, 2 * account_minimum_balance());
    assert!(account_account.data.is_empty());
}

#[test]
fn test_overflow() {
    let program_id = spl_token::id();
    let account_key = Pubkey::new_unique();
    let mut account_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let account2_key = Pubkey::new_unique();
    let mut account2_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let owner_key = Pubkey::new_unique();
    let mut owner_account = SolanaAccount::default();
    let owner2_key = Pubkey::new_unique();
    let mut owner2_account = SolanaAccount::default();
    let mint_owner_key = Pubkey::new_unique();
    let mut mint_owner_account = SolanaAccount::default();
    let mint_key = Pubkey::new_unique();
    let mut mint_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let mut rent_sysvar = rent_sysvar();

    // create new mint with owner
    do_process_instruction(
        initialize_mint(&program_id, &mint_key, &mint_owner_key, None, 2).unwrap(),
        vec![&mut mint_account, &mut rent_sysvar],
        &[Check::success()],
    )
    .unwrap();

    // create an account
    do_process_instruction(
        initialize_account(&program_id, &account_key, &mint_key, &owner_key).unwrap(),
        vec![
            &mut account_account,
            &mut mint_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();

    // create another account
    do_process_instruction(
        initialize_account(&program_id, &account2_key, &mint_key, &owner2_key).unwrap(),
        vec![
            &mut account2_account,
            &mut mint_account,
            &mut owner2_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();

    // mint the max to an account
    do_process_instruction(
        mint_to(
            &program_id,
            &mint_key,
            &account_key,
            &mint_owner_key,
            &[],
            u64::MAX,
        )
        .unwrap(),
        vec![
            &mut mint_account,
            &mut account_account,
            &mut mint_owner_account,
        ],
        &[
            Check::success(),
            Check::account(&account_key)
                .data_slice(64, &u64::MAX.to_le_bytes())
                .build(),
        ],
    )
    .unwrap();
    let account = Account::unpack_unchecked(&account_account.data).unwrap();
    assert_eq!(account.amount, u64::MAX);

    // attempt to mint one more to account
    assert_eq!(
        Err(TokenError::Overflow.into()),
        do_process_instruction(
            mint_to(
                &program_id,
                &mint_key,
                &account_key,
                &mint_owner_key,
                &[],
                1,
            )
            .unwrap(),
            vec![
                &mut mint_account,
                &mut account_account,
                &mut mint_owner_account,
            ],
            &[
                Check::err(TokenError::Overflow.into()),
                Check::account(&account_key)
                    .data_slice(64, &u64::MAX.to_le_bytes())
                    .build(),
            ],
        )
    );
    let account = Account::unpack_unchecked(&account_account.data).unwrap();
    assert_eq!(account.amount, u64::MAX);

    // attempt to mint one more to the other account
    assert_eq!(
        Err(TokenError::Overflow.into()),
        do_process_instruction(
            mint_to(
                &program_id,
                &mint_key,
                &account2_key,
                &mint_owner_key,
                &[],
                1,
            )
            .unwrap(),
            vec![
                &mut mint_account,
                &mut account2_account,
                &mut mint_owner_account,
            ],
            &[Check::err(TokenError::Overflow.into())],
        )
    );

    // burn some of the supply
    do_process_instruction(
        burn(&program_id, &account_key, &mint_key, &owner_key, &[], 100).unwrap(),
        vec![&mut account_account, &mut mint_account, &mut owner_account],
        &[
            Check::success(),
            Check::account(&account_key)
                .data_slice(64, &(u64::MAX - 100).to_le_bytes())
                .build(),
        ],
    )
    .unwrap();
    let account = Account::unpack_unchecked(&account_account.data).unwrap();
    assert_eq!(account.amount, u64::MAX - 100);

    do_process_instruction(
        mint_to(
            &program_id,
            &mint_key,
            &account_key,
            &mint_owner_key,
            &[],
            100,
        )
        .unwrap(),
        vec![
            &mut mint_account,
            &mut account_account,
            &mut mint_owner_account,
        ],
        &[
            Check::success(),
            Check::account(&account_key)
                .data_slice(64, &u64::MAX.to_le_bytes())
                .build(),
        ],
    )
    .unwrap();
    let account = Account::unpack_unchecked(&account_account.data).unwrap();
    assert_eq!(account.amount, u64::MAX);
}

#[test]
fn test_frozen() {
    let program_id = spl_token::id();
    let account_key = Pubkey::new_unique();
    let mut account_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let account2_key = Pubkey::new_unique();
    let mut account2_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let owner_key = Pubkey::new_unique();
    let mut owner_account = SolanaAccount::default();
    let mint_key = Pubkey::new_unique();
    let mut mint_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let mut rent_sysvar = rent_sysvar();

    // create new mint and fund first account
    do_process_instruction(
        initialize_mint(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
        vec![&mut mint_account, &mut rent_sysvar],
        &[Check::success()],
    )
    .unwrap();

    // create account
    do_process_instruction(
        initialize_account(&program_id, &account_key, &mint_key, &owner_key).unwrap(),
        vec![
            &mut account_account,
            &mut mint_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();

    // create another account
    do_process_instruction(
        initialize_account(&program_id, &account2_key, &mint_key, &owner_key).unwrap(),
        vec![
            &mut account2_account,
            &mut mint_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();

    // fund first account
    do_process_instruction(
        mint_to(&program_id, &mint_key, &account_key, &owner_key, &[], 1000).unwrap(),
        vec![&mut mint_account, &mut account_account, &mut owner_account],
        &[Check::success()],
    )
    .unwrap();

    // no transfer if either account is frozen
    let mut account = Account::unpack_unchecked(&account2_account.data).unwrap();
    account.state = AccountState::Frozen;
    Account::pack(account, &mut account2_account.data).unwrap();
    assert_eq!(
        Err(TokenError::AccountFrozen.into()),
        do_process_instruction(
            transfer(
                &program_id,
                &account_key,
                &account2_key,
                &owner_key,
                &[],
                500,
            )
            .unwrap(),
            vec![
                &mut account_account,
                &mut account2_account,
                &mut owner_account,
            ],
            &[Check::err(TokenError::AccountFrozen.into())],
        )
    );

    let mut account = Account::unpack_unchecked(&account_account.data).unwrap();
    account.state = AccountState::Initialized;
    Account::pack(account, &mut account_account.data).unwrap();
    let mut account = Account::unpack_unchecked(&account2_account.data).unwrap();
    account.state = AccountState::Frozen;
    Account::pack(account, &mut account2_account.data).unwrap();
    assert_eq!(
        Err(TokenError::AccountFrozen.into()),
        do_process_instruction(
            transfer(
                &program_id,
                &account_key,
                &account2_key,
                &owner_key,
                &[],
                500,
            )
            .unwrap(),
            vec![
                &mut account_account,
                &mut account2_account,
                &mut owner_account,
            ],
            &[Check::err(TokenError::AccountFrozen.into())],
        )
    );

    // no approve if account is frozen
    let mut account = Account::unpack_unchecked(&account_account.data).unwrap();
    account.state = AccountState::Frozen;
    Account::pack(account, &mut account_account.data).unwrap();
    let delegate_key = Pubkey::new_unique();
    let mut delegate_account = SolanaAccount::default();
    assert_eq!(
        Err(TokenError::AccountFrozen.into()),
        do_process_instruction(
            approve(
                &program_id,
                &account_key,
                &delegate_key,
                &owner_key,
                &[],
                100
            )
            .unwrap(),
            vec![
                &mut account_account,
                &mut delegate_account,
                &mut owner_account,
            ],
            &[Check::err(TokenError::AccountFrozen.into())],
        )
    );

    // no revoke if account is frozen
    let mut account = Account::unpack_unchecked(&account_account.data).unwrap();
    account.delegate = COption::Some(delegate_key);
    account.delegated_amount = 100;
    Account::pack(account, &mut account_account.data).unwrap();
    assert_eq!(
        Err(TokenError::AccountFrozen.into()),
        do_process_instruction(
            revoke(&program_id, &account_key, &owner_key, &[]).unwrap(),
            vec![&mut account_account, &mut owner_account],
            &[Check::err(TokenError::AccountFrozen.into())],
        )
    );

    // no set authority if account is frozen
    let new_owner_key = Pubkey::new_unique();
    assert_eq!(
        Err(TokenError::AccountFrozen.into()),
        do_process_instruction(
            set_authority(
                &program_id,
                &account_key,
                Some(&new_owner_key),
                AuthorityType::AccountOwner,
                &owner_key,
                &[]
            )
            .unwrap(),
            vec![&mut account_account, &mut owner_account,],
            &[Check::err(TokenError::AccountFrozen.into())],
        )
    );

    // no mint_to if destination account is frozen
    assert_eq!(
        Err(TokenError::AccountFrozen.into()),
        do_process_instruction(
            mint_to(&program_id, &mint_key, &account_key, &owner_key, &[], 100).unwrap(),
            vec![&mut mint_account, &mut account_account, &mut owner_account,],
            &[Check::err(TokenError::AccountFrozen.into())],
        )
    );

    // no burn if account is frozen
    assert_eq!(
        Err(TokenError::AccountFrozen.into()),
        do_process_instruction(
            burn(&program_id, &account_key, &mint_key, &owner_key, &[], 100).unwrap(),
            vec![&mut account_account, &mut mint_account, &mut owner_account],
            &[Check::err(TokenError::AccountFrozen.into())],
        )
    );
}

#[test]
fn test_freeze_thaw_dups() {
    let program_id = spl_token::id();
    let account1_key = Pubkey::new_unique();
    let mut account1_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let account1_info: AccountInfo = (&account1_key, true, &mut account1_account).into();
    let owner_key = Pubkey::new_unique();
    let mint_key = Pubkey::new_unique();
    let mut mint_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let mint_info: AccountInfo = (&mint_key, true, &mut mint_account).into();
    let rent_key = rent::id();
    let mut rent_sysvar = rent_sysvar();
    let rent_info: AccountInfo = (&rent_key, false, &mut rent_sysvar).into();

    // create mint
    do_process_instruction_dups(
        initialize_mint(&program_id, &mint_key, &owner_key, Some(&account1_key), 2).unwrap(),
        vec![mint_info.clone(), rent_info.clone()],
        &[Check::success()],
    )
    .unwrap();

    // create account
    do_process_instruction_dups(
        initialize_account(&program_id, &account1_key, &mint_key, &account1_key).unwrap(),
        vec![
            account1_info.clone(),
            mint_info.clone(),
            account1_info.clone(),
            rent_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // freeze where mint freeze_authority is account
    do_process_instruction_dups(
        freeze_account(&program_id, &account1_key, &mint_key, &account1_key, &[]).unwrap(),
        vec![
            account1_info.clone(),
            mint_info.clone(),
            account1_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();

    // thaw where mint freeze_authority is account
    let mut account = Account::unpack_unchecked(&account1_info.data.borrow()).unwrap();
    account.state = AccountState::Frozen;
    Account::pack(account, &mut account1_info.data.borrow_mut()).unwrap();
    do_process_instruction_dups(
        thaw_account(&program_id, &account1_key, &mint_key, &account1_key, &[]).unwrap(),
        vec![
            account1_info.clone(),
            mint_info.clone(),
            account1_info.clone(),
        ],
        &[Check::success()],
    )
    .unwrap();
}

#[test]
fn test_freeze_account() {
    let program_id = spl_token::id();
    let account_key = Pubkey::new_unique();
    let mut account_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let account_owner_key = Pubkey::new_unique();
    let mut account_owner_account = SolanaAccount::default();
    let owner_key = Pubkey::new_unique();
    let mut owner_account = SolanaAccount::default();
    let owner2_key = Pubkey::new_unique();
    let mut owner2_account = SolanaAccount::default();
    let mint_key = Pubkey::new_unique();
    let mut mint_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let mut rent_sysvar = rent_sysvar();

    // create new mint with owner different from account owner
    do_process_instruction(
        initialize_mint(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
        vec![&mut mint_account, &mut rent_sysvar],
        &[Check::success()],
    )
    .unwrap();

    // create account
    do_process_instruction(
        initialize_account(&program_id, &account_key, &mint_key, &account_owner_key).unwrap(),
        vec![
            &mut account_account,
            &mut mint_account,
            &mut account_owner_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();

    // mint to account
    do_process_instruction(
        mint_to(&program_id, &mint_key, &account_key, &owner_key, &[], 1000).unwrap(),
        vec![&mut mint_account, &mut account_account, &mut owner_account],
        &[Check::success()],
    )
    .unwrap();

    // mint cannot freeze
    assert_eq!(
        Err(TokenError::MintCannotFreeze.into()),
        do_process_instruction(
            freeze_account(&program_id, &account_key, &mint_key, &owner_key, &[]).unwrap(),
            vec![&mut account_account, &mut mint_account, &mut owner_account],
            &[Check::err(TokenError::MintCannotFreeze.into())],
        )
    );

    // missing freeze_authority
    let mut mint = Mint::unpack_unchecked(&mint_account.data).unwrap();
    mint.freeze_authority = COption::Some(owner_key);
    Mint::pack(mint, &mut mint_account.data).unwrap();
    assert_eq!(
        Err(TokenError::OwnerMismatch.into()),
        do_process_instruction(
            freeze_account(&program_id, &account_key, &mint_key, &owner2_key, &[]).unwrap(),
            vec![&mut account_account, &mut mint_account, &mut owner2_account],
            &[Check::err(TokenError::OwnerMismatch.into())],
        )
    );

    // check explicit thaw
    assert_eq!(
        Err(TokenError::InvalidState.into()),
        do_process_instruction(
            thaw_account(&program_id, &account_key, &mint_key, &owner2_key, &[]).unwrap(),
            vec![&mut account_account, &mut mint_account, &mut owner2_account],
            &[Check::err(TokenError::InvalidState.into())],
        )
    );

    // freeze
    do_process_instruction(
        freeze_account(&program_id, &account_key, &mint_key, &owner_key, &[]).unwrap(),
        vec![&mut account_account, &mut mint_account, &mut owner_account],
        &[
            Check::success(),
            Check::account(&account_key)
                .data_slice(108, &[AccountState::Frozen as u8])
                .build(),
        ],
    )
    .unwrap();
    let account = Account::unpack_unchecked(&account_account.data).unwrap();
    assert_eq!(account.state, AccountState::Frozen);

    // check explicit freeze
    assert_eq!(
        Err(TokenError::InvalidState.into()),
        do_process_instruction(
            freeze_account(&program_id, &account_key, &mint_key, &owner_key, &[]).unwrap(),
            vec![&mut account_account, &mut mint_account, &mut owner_account],
            &[Check::err(TokenError::InvalidState.into())],
        )
    );

    // check thaw authority
    assert_eq!(
        Err(TokenError::OwnerMismatch.into()),
        do_process_instruction(
            thaw_account(&program_id, &account_key, &mint_key, &owner2_key, &[]).unwrap(),
            vec![&mut account_account, &mut mint_account, &mut owner2_account],
            &[Check::err(TokenError::OwnerMismatch.into())],
        )
    );

    // thaw
    do_process_instruction(
        thaw_account(&program_id, &account_key, &mint_key, &owner_key, &[]).unwrap(),
        vec![&mut account_account, &mut mint_account, &mut owner_account],
        &[
            Check::success(),
            Check::account(&account_key)
                .data_slice(108, &[AccountState::Initialized as u8])
                .build(),
        ],
    )
    .unwrap();
    let account = Account::unpack_unchecked(&account_account.data).unwrap();
    assert_eq!(account.state, AccountState::Initialized);
}

#[test]
fn test_initialize_account2_and_3() {
    let program_id = spl_token::id();
    let account_key = Pubkey::new_unique();
    let mut account_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let mut account2_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let mut account3_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let owner_key = Pubkey::new_unique();
    let mut owner_account = SolanaAccount::default();
    let mint_key = Pubkey::new_unique();
    let mut mint_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let mut rent_sysvar = rent_sysvar();

    // create mint
    do_process_instruction(
        initialize_mint(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
        vec![&mut mint_account, &mut rent_sysvar],
        &[Check::success()],
    )
    .unwrap();

    do_process_instruction(
        initialize_account(&program_id, &account_key, &mint_key, &owner_key).unwrap(),
        vec![
            &mut account_account,
            &mut mint_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();

    do_process_instruction(
        initialize_account2(&program_id, &account_key, &mint_key, &owner_key).unwrap(),
        vec![&mut account2_account, &mut mint_account, &mut rent_sysvar],
        &[Check::success()],
    )
    .unwrap();

    assert_eq!(account_account, account2_account);

    do_process_instruction(
        initialize_account3(&program_id, &account_key, &mint_key, &owner_key).unwrap(),
        vec![&mut account3_account, &mut mint_account],
        &[Check::success()],
    )
    .unwrap();

    assert_eq!(account_account, account3_account);
}

#[test]
fn test_sync_native() {
    let program_id = spl_token::id();
    let mint_key = Pubkey::new_unique();
    let mut mint_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let native_account_key = Pubkey::new_unique();
    let lamports = 40;
    let mut native_account = SolanaAccount::new(
        account_minimum_balance() + lamports,
        Account::get_packed_len(),
        &program_id,
    );
    let non_native_account_key = Pubkey::new_unique();
    let mut non_native_account = SolanaAccount::new(
        account_minimum_balance() + 50,
        Account::get_packed_len(),
        &program_id,
    );

    let owner_key = Pubkey::new_unique();
    let mut owner_account = SolanaAccount::default();
    let mut rent_sysvar = rent_sysvar();

    // initialize non-native mint
    do_process_instruction(
        initialize_mint(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
        vec![&mut mint_account, &mut rent_sysvar],
        &[Check::success()],
    )
    .unwrap();

    // initialize non-native account
    do_process_instruction(
        initialize_account(&program_id, &non_native_account_key, &mint_key, &owner_key).unwrap(),
        vec![
            &mut non_native_account,
            &mut mint_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[
            Check::success(),
            Check::account(&non_native_account_key)
                .data_slice(109, &[0, 0, 0, 0])
                .build(),
            Check::account(&non_native_account_key)
                .data_slice(64, &0u64.to_le_bytes())
                .build(),
        ],
    )
    .unwrap();

    let account = Account::unpack_unchecked(&non_native_account.data).unwrap();
    assert!(!account.is_native());
    assert_eq!(account.amount, 0);

    // fail sync non-native
    assert_eq!(
        Err(TokenError::NonNativeNotSupported.into()),
        do_process_instruction(
            sync_native(&program_id, &non_native_account_key,).unwrap(),
            vec![&mut non_native_account],
            &[Check::err(TokenError::NonNativeNotSupported.into())],
        )
    );

    // fail sync uninitialized
    assert_eq!(
        Err(ProgramError::UninitializedAccount),
        do_process_instruction(
            sync_native(&program_id, &native_account_key,).unwrap(),
            vec![&mut native_account],
            &[Check::err(ProgramError::UninitializedAccount)],
        )
    );

    // wrap native account
    do_process_instruction(
        initialize_account(
            &program_id,
            &native_account_key,
            &spl_token::native_mint::id(),
            &owner_key,
        )
        .unwrap(),
        vec![
            &mut native_account,
            &mut mint_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();

    // fail sync, not owned by program
    let not_program_id = Pubkey::new_unique();
    native_account.owner = not_program_id;
    assert_eq!(
        Err(ProgramError::IncorrectProgramId),
        do_process_instruction(
            sync_native(&program_id, &native_account_key,).unwrap(),
            vec![&mut native_account],
            &[
                Check::err(ProgramError::IncorrectProgramId),
                Check::account(&native_account_key)
                    .data_slice(109, &[1, 0, 0, 0])
                    .build(),
                Check::account(&native_account_key)
                    .data_slice(64, &lamports.to_le_bytes())
                    .build()
            ],
        )
    );
    native_account.owner = program_id;

    let account = Account::unpack_unchecked(&native_account.data).unwrap();
    assert!(account.is_native());
    assert_eq!(account.amount, lamports);

    // sync, no change
    do_process_instruction(
        sync_native(&program_id, &native_account_key).unwrap(),
        vec![&mut native_account],
        &[
            Check::success(),
            Check::account(&native_account_key)
                .data_slice(64, &lamports.to_le_bytes())
                .build(),
        ],
    )
    .unwrap();
    let account = Account::unpack_unchecked(&native_account.data).unwrap();
    assert_eq!(account.amount, lamports);

    // transfer sol
    let new_lamports = lamports + 50;
    native_account.lamports = account_minimum_balance() + new_lamports;

    // success sync
    do_process_instruction(
        sync_native(&program_id, &native_account_key).unwrap(),
        vec![&mut native_account],
        &[
            Check::success(),
            Check::account(&native_account_key)
                .data_slice(64, &new_lamports.to_le_bytes())
                .build(),
        ],
    )
    .unwrap();
    let account = Account::unpack_unchecked(&native_account.data).unwrap();
    assert_eq!(account.amount, new_lamports);

    // reduce sol
    native_account.lamports -= 1;

    // fail sync
    assert_eq!(
        Err(TokenError::InvalidState.into()),
        do_process_instruction(
            sync_native(&program_id, &native_account_key,).unwrap(),
            vec![&mut native_account],
            &[Check::err(TokenError::InvalidState.into())],
        )
    );
}

#[test]
#[serial]
fn test_get_account_data_size() {
    // see integration tests for return-data validity
    let program_id = spl_token::id();
    let owner_key = Pubkey::new_unique();
    let mut rent_sysvar = rent_sysvar();
    let mut mint_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let mint_key = Pubkey::new_unique();
    // fail if an invalid mint is passed in
    assert_eq!(
        Err(TokenError::InvalidMint.into()),
        do_process_instruction(
            get_account_data_size(&program_id, &mint_key).unwrap(),
            vec![&mut mint_account],
            &[Check::err(TokenError::InvalidMint.into())],
        )
    );

    do_process_instruction(
        initialize_mint(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
        vec![&mut mint_account, &mut rent_sysvar],
        &[Check::success()],
    )
    .unwrap();

    do_process_instruction(
        get_account_data_size(&program_id, &mint_key).unwrap(),
        vec![&mut mint_account],
        &[
            Check::success(),
            Check::return_data(&Account::LEN.to_le_bytes()),
        ],
    )
    .unwrap();
}

#[test]
fn test_initialize_immutable_owner() {
    let program_id = spl_token::id();
    let account_key = Pubkey::new_unique();
    let mut account_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let owner_key = Pubkey::new_unique();
    let mut owner_account = SolanaAccount::default();
    let mint_key = Pubkey::new_unique();
    let mut mint_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let mut rent_sysvar = rent_sysvar();

    // create mint
    do_process_instruction(
        initialize_mint(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
        vec![&mut mint_account, &mut rent_sysvar],
        &[Check::success()],
    )
    .unwrap();

    // success initialize immutable
    do_process_instruction(
        initialize_immutable_owner(&program_id, &account_key).unwrap(),
        vec![&mut account_account],
        &[Check::success()],
    )
    .unwrap();

    // create account
    do_process_instruction(
        initialize_account(&program_id, &account_key, &mint_key, &owner_key).unwrap(),
        vec![
            &mut account_account,
            &mut mint_account,
            &mut owner_account,
            &mut rent_sysvar,
        ],
        &[Check::success()],
    )
    .unwrap();

    // fail post-init
    assert_eq!(
        Err(TokenError::AlreadyInUse.into()),
        do_process_instruction(
            initialize_immutable_owner(&program_id, &account_key).unwrap(),
            vec![&mut account_account],
            &[Check::err(TokenError::AlreadyInUse.into())],
        )
    );
}

#[test]
#[serial]
fn test_amount_to_ui_amount() {
    let program_id = spl_token::id();
    let owner_key = Pubkey::new_unique();
    let mint_key = Pubkey::new_unique();
    let mut mint_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let mut rent_sysvar = rent_sysvar();

    // fail if an invalid mint is passed in
    assert_eq!(
        Err(TokenError::InvalidMint.into()),
        do_process_instruction(
            amount_to_ui_amount(&program_id, &mint_key, 110).unwrap(),
            vec![&mut mint_account],
            &[Check::err(TokenError::InvalidMint.into())],
        )
    );

    // create mint
    do_process_instruction(
        initialize_mint(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
        vec![&mut mint_account, &mut rent_sysvar],
        &[Check::success()],
    )
    .unwrap();

    do_process_instruction(
        amount_to_ui_amount(&program_id, &mint_key, 23).unwrap(),
        vec![&mut mint_account],
        &[Check::success(), Check::return_data("0.23".as_bytes())],
    )
    .unwrap();

    do_process_instruction(
        amount_to_ui_amount(&program_id, &mint_key, 110).unwrap(),
        vec![&mut mint_account],
        &[Check::success(), Check::return_data("1.1".as_bytes())],
    )
    .unwrap();

    do_process_instruction(
        amount_to_ui_amount(&program_id, &mint_key, 4200).unwrap(),
        vec![&mut mint_account],
        &[Check::success(), Check::return_data("42".as_bytes())],
    )
    .unwrap();

    do_process_instruction(
        amount_to_ui_amount(&program_id, &mint_key, 0).unwrap(),
        vec![&mut mint_account],
        &[Check::success(), Check::return_data("0".as_bytes())],
    )
    .unwrap();
}

#[test]
#[serial]
fn test_ui_amount_to_amount() {
    let program_id = spl_token::id();
    let owner_key = Pubkey::new_unique();
    let mint_key = Pubkey::new_unique();
    let mut mint_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let mut rent_sysvar = rent_sysvar();

    // fail if an invalid mint is passed in
    assert_eq!(
        Err(TokenError::InvalidMint.into()),
        do_process_instruction(
            ui_amount_to_amount(&program_id, &mint_key, "1.1").unwrap(),
            vec![&mut mint_account],
            &[Check::err(TokenError::InvalidMint.into())],
        )
    );

    // create mint
    do_process_instruction(
        initialize_mint(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
        vec![&mut mint_account, &mut rent_sysvar],
        &[Check::success()],
    )
    .unwrap();

    do_process_instruction(
        ui_amount_to_amount(&program_id, &mint_key, "0.23").unwrap(),
        vec![&mut mint_account],
        &[Check::success(), Check::return_data(&23u64.to_le_bytes())],
    )
    .unwrap();

    do_process_instruction(
        ui_amount_to_amount(&program_id, &mint_key, "0.20").unwrap(),
        vec![&mut mint_account],
        &[Check::success(), Check::return_data(&20u64.to_le_bytes())],
    )
    .unwrap();

    do_process_instruction(
        ui_amount_to_amount(&program_id, &mint_key, "0.2000").unwrap(),
        vec![&mut mint_account],
        &[Check::success(), Check::return_data(&20u64.to_le_bytes())],
    )
    .unwrap();

    do_process_instruction(
        ui_amount_to_amount(&program_id, &mint_key, ".20").unwrap(),
        vec![&mut mint_account],
        &[Check::success(), Check::return_data(&20u64.to_le_bytes())],
    )
    .unwrap();

    do_process_instruction(
        ui_amount_to_amount(&program_id, &mint_key, "1.1").unwrap(),
        vec![&mut mint_account],
        &[Check::success(), Check::return_data(&110u64.to_le_bytes())],
    )
    .unwrap();

    do_process_instruction(
        ui_amount_to_amount(&program_id, &mint_key, "1.10").unwrap(),
        vec![&mut mint_account],
        &[Check::success(), Check::return_data(&110u64.to_le_bytes())],
    )
    .unwrap();

    do_process_instruction(
        ui_amount_to_amount(&program_id, &mint_key, "42").unwrap(),
        vec![&mut mint_account],
        &[Check::success(), Check::return_data(&4200u64.to_le_bytes())],
    )
    .unwrap();

    do_process_instruction(
        ui_amount_to_amount(&program_id, &mint_key, "42.").unwrap(),
        vec![&mut mint_account],
        &[Check::success(), Check::return_data(&4200u64.to_le_bytes())],
    )
    .unwrap();

    do_process_instruction(
        ui_amount_to_amount(&program_id, &mint_key, "0").unwrap(),
        vec![&mut mint_account],
        &[Check::success(), Check::return_data(&0u64.to_le_bytes())],
    )
    .unwrap();

    // fail if invalid ui_amount passed in
    assert_eq!(
        Err(ProgramError::InvalidArgument),
        do_process_instruction(
            ui_amount_to_amount(&program_id, &mint_key, "").unwrap(),
            vec![&mut mint_account],
            &[Check::err(ProgramError::InvalidArgument)],
        )
    );
    assert_eq!(
        Err(ProgramError::InvalidArgument),
        do_process_instruction(
            ui_amount_to_amount(&program_id, &mint_key, ".").unwrap(),
            vec![&mut mint_account],
            &[Check::err(ProgramError::InvalidArgument)],
        )
    );
    assert_eq!(
        Err(ProgramError::InvalidArgument),
        do_process_instruction(
            ui_amount_to_amount(&program_id, &mint_key, "0.111").unwrap(),
            vec![&mut mint_account],
            &[Check::err(ProgramError::InvalidArgument)],
        )
    );
    assert_eq!(
        Err(ProgramError::InvalidArgument),
        do_process_instruction(
            ui_amount_to_amount(&program_id, &mint_key, "0.t").unwrap(),
            vec![&mut mint_account],
            &[Check::err(ProgramError::InvalidArgument)],
        )
    );
}
