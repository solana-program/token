#![cfg(feature = "test-sbf")]

//! Program state processor tests

use {
    mollusk_svm::{
        result::{Check, InstructionResult},
        Mollusk,
    },
    solana_sdk::{
        account::{create_account_for_test, Account as SolanaAccount, AccountSharedData},
        instruction::Instruction,
        program_error::ProgramError,
        program_option::COption,
        program_pack::Pack,
        pubkey::Pubkey,
        rent::Rent,
    },
    spl_token::{
        error::TokenError,
        instruction::{
            approve, initialize_account, initialize_mint, initialize_mint2, initialize_multisig,
            mint_to, transfer, transfer_checked,
        },
        state::{Account, AccountState, Mint, Multisig},
    },
    std::collections::HashSet,
};

/// The name of the token program `.so` file.
const TOKEN_PROGRAM_NAME: &str = "spl_token";

/// The ID of the token program used in the instruction constructors.
///
/// In general this should be the same as the `spl_token::id()` constant, since
/// the instruction construction functions are designed to work with the
/// `spl_token`. This value then is replaced by [`TARGET_TOKEN_PROGRAM_ID`] when
/// the instruction is processed by `mollusk` helper.
const INSTRUCTION_TOKEN_PROGRAM_ID: Pubkey = spl_token::id();

/// The ID of the token program that will execute the instruction.
const TARGET_TOKEN_PROGRAM_ID: Pubkey = spl_token::id();

/// A tuple of an instruction and the accounts it references.
type InstructionPack<'a> = (Instruction, Vec<&'a SolanaAccount>);

/// Process a list of instructions using mollusk.
fn do_process_instructions(
    instructions: &[InstructionPack],
    checks: &[Check],
) -> InstructionResult {
    do_process_instructions_with_pre_instructions(None, instructions, checks)
}

/// Process a list of instructions using mollusk with a pre-defined set of
/// "setup" instructions.
fn do_process_instructions_with_pre_instructions(
    pre_instructions: Option<&[InstructionPack]>,
    instructions: &[InstructionPack],
    checks: &[Check],
) -> InstructionResult {
    // Track the accounts that have been set up.
    let mut cached_accounts = HashSet::new();
    // List of instructions to process.
    let mut tx_instructions = Vec::new();
    // List of accounts to process.
    let mut tx_accounts = Vec::new();

    // Process pre-instructions.
    if let Some(pre_instructions) = pre_instructions {
        pre_instructions.iter().for_each(|(instruction, accounts)| {
            instruction
                .accounts
                .iter()
                .zip(accounts)
                .map(|(account_meta, account)| {
                    (
                        account_meta.pubkey,
                        AccountSharedData::from((*account).clone()),
                    )
                })
                .for_each(|(pubkey, account)| {
                    if !cached_accounts.contains(&pubkey) {
                        cached_accounts.insert(pubkey);
                        tx_accounts.push((pubkey, account));
                    }
                });
            let mut ix = instruction.clone();
            ix.program_id = TARGET_TOKEN_PROGRAM_ID;
            tx_instructions.push(ix);
        });
    }

    // Process instructions.
    instructions.iter().for_each(|(instruction, accounts)| {
        instruction
            .accounts
            .iter()
            .zip(accounts)
            .map(|(account_meta, account)| {
                (
                    account_meta.pubkey,
                    AccountSharedData::from((*account).clone()),
                )
            })
            .for_each(|(pubkey, account)| {
                if !cached_accounts.contains(&pubkey) {
                    cached_accounts.insert(pubkey);
                    tx_accounts.push((pubkey, account));
                }
            });
        let mut ix = instruction.clone();
        ix.program_id = TARGET_TOKEN_PROGRAM_ID;
        tx_instructions.push(ix);
    });

    let mollusk = Mollusk::new(&TARGET_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_NAME);
    mollusk.process_and_validate_instruction_chain(
        tx_instructions.as_slice(),
        tx_accounts.as_slice(),
        checks,
    )
}

fn account_minimum_balance() -> u64 {
    Rent::default().minimum_balance(Account::get_packed_len())
}

fn mint_minimum_balance() -> u64 {
    Rent::default().minimum_balance(Mint::get_packed_len())
}

fn multisig_minimum_balance() -> u64 {
    Rent::default().minimum_balance(Multisig::get_packed_len())
}

fn rent_sysvar() -> SolanaAccount {
    create_account_for_test(&Rent::default())
}

#[test]
fn test_initialize_mint() {
    let program_id = TARGET_TOKEN_PROGRAM_ID;
    let owner_key = Pubkey::new_unique();
    let mint_key = Pubkey::new_unique();
    let mut mint_account = SolanaAccount::new(42, Mint::get_packed_len(), &program_id);
    let mint2_key = Pubkey::new_unique();
    let mint2_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let rent_sysvar = rent_sysvar();

    // mint is not rent exempt
    do_process_instructions(
        &[(
            initialize_mint(
                &INSTRUCTION_TOKEN_PROGRAM_ID,
                &mint_key,
                &owner_key,
                None,
                2,
            )
            .unwrap(),
            vec![&mint_account, &rent_sysvar],
        )],
        &[Check::err(TokenError::NotRentExempt.into())],
    );

    mint_account.lamports = mint_minimum_balance();

    // create new mint
    do_process_instructions(
        &[(
            initialize_mint(
                &INSTRUCTION_TOKEN_PROGRAM_ID,
                &mint_key,
                &owner_key,
                None,
                2,
            )
            .unwrap(),
            vec![&mint_account, &rent_sysvar],
        )],
        &[Check::success()],
    );

    // create twice
    do_process_instructions(
        &[
            (
                initialize_mint(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &mint_key,
                    &owner_key,
                    None,
                    2,
                )
                .unwrap(),
                vec![&mint_account, &rent_sysvar],
            ),
            (
                initialize_mint(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &mint_key,
                    &owner_key,
                    None,
                    2,
                )
                .unwrap(),
                vec![&mint_account, &rent_sysvar],
            ),
        ],
        &[Check::err(TokenError::AlreadyInUse.into())],
    );

    // create another mint that can freeze
    do_process_instructions(
        &[(
            initialize_mint(
                &INSTRUCTION_TOKEN_PROGRAM_ID,
                &mint2_key,
                &owner_key,
                Some(&owner_key),
                2,
            )
            .unwrap(),
            vec![&mint2_account, &rent_sysvar],
        )],
        &[
            // Account successfully re-initialized.
            Check::success(),
            // mint authority is set
            Check::account(&mint2_key)
                .data_slice(46, &[1, 0, 0, 0])
                .build(),
            // mint authority matches owner
            Check::account(&mint2_key)
                .data_slice(50, owner_key.as_ref())
                .build(),
        ],
    );
}

#[test]
fn test_initialize_mint2() {
    let program_id = TARGET_TOKEN_PROGRAM_ID;
    let owner_key = Pubkey::new_unique();
    let mint_key = Pubkey::new_unique();
    let mut mint_account = SolanaAccount::new(42, Mint::get_packed_len(), &program_id);
    let mint2_key = Pubkey::new_unique();
    let mut mint2_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);

    // mint is not rent exempt
    do_process_instructions(
        &[(
            initialize_mint2(
                &INSTRUCTION_TOKEN_PROGRAM_ID,
                &mint_key,
                &owner_key,
                None,
                2,
            )
            .unwrap(),
            vec![&mint_account],
        )],
        &[Check::err(TokenError::NotRentExempt.into())],
    );

    mint_account.lamports = mint_minimum_balance();

    // create new mint
    do_process_instructions(
        &[(
            initialize_mint2(
                &INSTRUCTION_TOKEN_PROGRAM_ID,
                &mint_key,
                &owner_key,
                None,
                2,
            )
            .unwrap(),
            vec![&mint_account],
        )],
        &[Check::success()],
    );

    // create twice
    do_process_instructions(
        &[
            (
                initialize_mint2(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &mint_key,
                    &owner_key,
                    None,
                    2,
                )
                .unwrap(),
                vec![&mint_account],
            ),
            (
                initialize_mint2(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &mint_key,
                    &owner_key,
                    None,
                    2,
                )
                .unwrap(),
                vec![&mint_account],
            ),
        ],
        &[Check::err(TokenError::AlreadyInUse.into())],
    );

    // create another mint that can freeze
    do_process_instructions(
        &[(
            initialize_mint2(
                &INSTRUCTION_TOKEN_PROGRAM_ID,
                &mint2_key,
                &owner_key,
                Some(&owner_key),
                2,
            )
            .unwrap(),
            vec![&mut mint2_account],
        )],
        &[
            // Account successfully re-initialized.
            Check::success(),
            // mint authority is set
            Check::account(&mint2_key)
                .data_slice(46, &[1, 0, 0, 0])
                .build(),
            // mint authority matches owner
            Check::account(&mint2_key)
                .data_slice(50, owner_key.as_ref())
                .build(),
        ],
    );
}

#[test]
fn test_initialize_mint_account() {
    let program_id = TARGET_TOKEN_PROGRAM_ID;
    let account_key = Pubkey::new_unique();
    let mut account_account = SolanaAccount::new(42, Account::get_packed_len(), &program_id);
    let owner_key = Pubkey::new_unique();
    let owner_account = SolanaAccount::default();
    let mint_key = Pubkey::new_unique();
    let mut mint_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let rent_sysvar = rent_sysvar();

    // account is not rent exempt
    do_process_instructions(
        &[(
            initialize_account(
                &INSTRUCTION_TOKEN_PROGRAM_ID,
                &account_key,
                &mint_key,
                &owner_key,
            )
            .unwrap(),
            vec![
                &account_account,
                &mint_account,
                &owner_account,
                &rent_sysvar,
            ],
        )],
        &[Check::err(TokenError::NotRentExempt.into())],
    );

    account_account.lamports = account_minimum_balance();

    // mint is not valid (not initialized)
    do_process_instructions(
        &[(
            initialize_account(
                &INSTRUCTION_TOKEN_PROGRAM_ID,
                &account_key,
                &mint_key,
                &owner_key,
            )
            .unwrap(),
            vec![
                &account_account,
                &mint_account,
                &owner_account,
                &rent_sysvar,
            ],
        )],
        &[Check::err(TokenError::InvalidMint.into())],
    );

    // create mint
    do_process_instructions(
        &[(
            initialize_mint(
                &INSTRUCTION_TOKEN_PROGRAM_ID,
                &mint_key,
                &owner_key,
                None,
                2,
            )
            .unwrap(),
            vec![&mint_account, &rent_sysvar],
        )],
        &[Check::success()],
    );

    // mint not owned by program
    let not_program_id = Pubkey::new_unique();
    mint_account.owner = not_program_id;

    do_process_instructions(
        &[(
            initialize_account(
                &INSTRUCTION_TOKEN_PROGRAM_ID,
                &account_key,
                &mint_key,
                &owner_key,
            )
            .unwrap(),
            vec![
                &account_account,
                &mint_account,
                &owner_account,
                &rent_sysvar,
            ],
        )],
        &[Check::err(ProgramError::IncorrectProgramId)],
    );

    mint_account.owner = program_id;

    // create account
    do_process_instructions(
        &[
            (
                initialize_mint(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &mint_key,
                    &owner_key,
                    None,
                    2,
                )
                .unwrap(),
                vec![&mint_account, &rent_sysvar],
            ),
            (
                initialize_account(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account_key,
                    &mint_key,
                    &owner_key,
                )
                .unwrap(),
                vec![
                    &account_account,
                    &mint_account,
                    &owner_account,
                    &rent_sysvar,
                ],
            ),
        ],
        &[Check::success()],
    );

    // create twice
    do_process_instructions(
        &[
            (
                initialize_mint(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &mint_key,
                    &owner_key,
                    None,
                    2,
                )
                .unwrap(),
                vec![&mint_account, &rent_sysvar],
            ),
            (
                initialize_account(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account_key,
                    &mint_key,
                    &owner_key,
                )
                .unwrap(),
                vec![
                    &account_account,
                    &mint_account,
                    &owner_account,
                    &rent_sysvar,
                ],
            ),
            (
                initialize_account(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account_key,
                    &mint_key,
                    &owner_key,
                )
                .unwrap(),
                vec![
                    &account_account,
                    &mint_account,
                    &owner_account,
                    &rent_sysvar,
                ],
            ),
        ],
        &[Check::err(TokenError::AlreadyInUse.into())],
    );
}

#[test]
fn test_transfer_dups() {
    let program_id = TARGET_TOKEN_PROGRAM_ID;
    let account1_key = Pubkey::new_unique();
    let account1_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    //let mut account1_info: AccountInfo = (&account1_key, true, &mut
    // account1_account).into();
    let account2_key = Pubkey::new_unique();
    let account2_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    //let mut account2_info: AccountInfo = (&account2_key, false, &mut
    // account2_account).into();
    let account3_key = Pubkey::new_unique();
    let account3_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    //let account3_info: AccountInfo = (&account3_key, false, &mut
    // account3_account).into();
    let account4_key = Pubkey::new_unique();
    let account4_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    //let account4_info: AccountInfo = (&account4_key, true, &mut
    // account4_account).into();
    let multisig_key = Pubkey::new_unique();
    let multisig_account = SolanaAccount::new(
        multisig_minimum_balance(),
        Multisig::get_packed_len(),
        &program_id,
    );
    //let multisig_info: AccountInfo = (&multisig_key, true, &mut
    // multisig_account).into();
    let owner_key = Pubkey::new_unique();
    let owner_account = SolanaAccount::default();
    //let owner_info: AccountInfo = (&owner_key, true, &mut owner_account).into();
    let mint_key = Pubkey::new_unique();
    let mint_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    //let mint_info: AccountInfo = (&mint_key, false, &mut mint_account).into();
    //let rent_key = solana_sdk::sysvar::rent::id();
    let rent_sysvar = rent_sysvar();
    //let rent_info: AccountInfo = (&rent_key, false, &mut rent_sysvar).into();

    let setup_instructions = vec![
        // create mint
        (
            initialize_mint(
                &INSTRUCTION_TOKEN_PROGRAM_ID,
                &mint_key,
                &owner_key,
                None,
                2,
            )
            .unwrap(),
            vec![&mint_account, &rent_sysvar],
        ),
        // create account
        (
            initialize_account(
                &INSTRUCTION_TOKEN_PROGRAM_ID,
                &account1_key,
                &mint_key,
                &account1_key,
            )
            .unwrap(),
            vec![
                &account1_account,
                &mint_account,
                &account1_account,
                &rent_sysvar,
            ],
        ),
        // create another account
        (
            initialize_account(
                &INSTRUCTION_TOKEN_PROGRAM_ID,
                &account2_key,
                &mint_key,
                &owner_key,
            )
            .unwrap(),
            vec![
                &account2_account,
                &mint_account,
                &owner_account,
                &rent_sysvar,
            ],
        ),
        // mint to account
        (
            mint_to(
                &INSTRUCTION_TOKEN_PROGRAM_ID,
                &mint_key,
                &account1_key,
                &owner_key,
                &[],
                1000,
            )
            .unwrap(),
            vec![&mint_account, &account1_account, &owner_account],
        ),
    ];

    // source-owner transfer
    do_process_instructions_with_pre_instructions(
        Some(&setup_instructions),
        &[(
            transfer(
                &program_id,
                &account1_key,
                &account2_key,
                &account1_key,
                &[],
                500,
            )
            .unwrap(),
            vec![&account1_account, &account2_account, &account1_account],
        )],
        &[Check::success()],
    );

    // source-owner TransferChecked
    do_process_instructions_with_pre_instructions(
        Some(&setup_instructions),
        &[(
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
                &account1_account,
                &mint_account,
                &account2_account,
                &account1_account,
            ],
        )],
        &[Check::success()],
    );

    // source-delegate transfer
    let delegate_key = Pubkey::new_unique();
    let delegate_account = SolanaAccount::default();

    let mut account = Account::unpack_unchecked(&account1_account.data).unwrap();
    account.state = AccountState::Initialized;
    account.mint = mint_key;
    account.amount = 1000;
    account.delegated_amount = 1000;
    account.delegate = COption::Some(delegate_key);
    account.owner = owner_key;
    let delegated_account_key = Pubkey::new_unique();
    let mut delegated_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    Account::pack(account, &mut delegated_account.data).unwrap();

    do_process_instructions_with_pre_instructions(
        Some(&setup_instructions),
        &[(
            transfer(
                &program_id,
                &delegated_account_key,
                &account2_key,
                &delegate_key,
                &[],
                500,
            )
            .unwrap(),
            vec![&delegated_account, &account2_account, &delegate_account],
        )],
        &[Check::success()],
    );

    // source-delegate TransferChecked
    do_process_instructions_with_pre_instructions(
        Some(&setup_instructions),
        &[(
            transfer_checked(
                &program_id,
                &delegated_account_key,
                &mint_key,
                &account2_key,
                &delegate_key,
                &[],
                500,
                2,
            )
            .unwrap(),
            vec![
                &delegated_account,
                &mint_account,
                &account2_account,
                &delegate_account,
            ],
        )],
        &[Check::success()],
    );

    // test destination-owner transfer
    do_process_instructions_with_pre_instructions(
        Some(&setup_instructions),
        &[
            (
                initialize_account(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account3_key,
                    &mint_key,
                    &account2_key,
                )
                .unwrap(),
                vec![
                    &account3_account,
                    &mint_account,
                    &account2_account,
                    &rent_sysvar,
                ],
            ),
            (
                mint_to(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &mint_key,
                    &account3_key,
                    &owner_key,
                    &[],
                    1000,
                )
                .unwrap(),
                vec![&mint_account, &account3_account, &owner_account],
            ),
            (
                transfer(
                    &program_id,
                    &account3_key,
                    &account2_key,
                    &account2_key,
                    &[],
                    500,
                )
                .unwrap(),
                vec![&account3_account, &account2_account, &account2_account],
            ),
        ],
        &[Check::success()],
    );

    // destination-owner TransferChecked
    do_process_instructions_with_pre_instructions(
        Some(&setup_instructions),
        &[
            (
                initialize_account(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account3_key,
                    &mint_key,
                    &account2_key,
                )
                .unwrap(),
                vec![
                    &account3_account,
                    &mint_account,
                    &account2_account,
                    &rent_sysvar,
                ],
            ),
            (
                mint_to(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &mint_key,
                    &account3_key,
                    &owner_key,
                    &[],
                    1000,
                )
                .unwrap(),
                vec![&mint_account, &account3_account, &owner_account],
            ),
            (
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
                    &account3_account,
                    &mint_account,
                    &account2_account,
                    &account2_account,
                ],
            ),
        ],
        &[Check::success()],
    );

    // test source-multisig signer
    do_process_instructions_with_pre_instructions(
        Some(&setup_instructions),
        &[
            (
                initialize_multisig(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &multisig_key,
                    &[&account4_key],
                    1,
                )
                .unwrap(),
                vec![&multisig_account, &rent_sysvar, &account4_account],
            ),
            (
                initialize_account(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account4_key,
                    &mint_key,
                    &multisig_key,
                )
                .unwrap(),
                vec![
                    &account4_account,
                    &mint_account,
                    &multisig_account,
                    &rent_sysvar,
                ],
            ),
            (
                mint_to(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &mint_key,
                    &account4_key,
                    &owner_key,
                    &[],
                    1000,
                )
                .unwrap(),
                vec![&mint_account, &account4_account, &owner_account],
            ),
        ],
        &[Check::success()],
    );

    // source-multisig-signer transfer
    do_process_instructions_with_pre_instructions(
        Some(&setup_instructions),
        &[
            (
                initialize_multisig(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &multisig_key,
                    &[&account4_key],
                    1,
                )
                .unwrap(),
                vec![&multisig_account, &rent_sysvar, &account4_account],
            ),
            (
                initialize_account(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account4_key,
                    &mint_key,
                    &multisig_key,
                )
                .unwrap(),
                vec![
                    &account4_account,
                    &mint_account,
                    &multisig_account,
                    &rent_sysvar,
                ],
            ),
            (
                mint_to(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &mint_key,
                    &account4_key,
                    &owner_key,
                    &[],
                    1000,
                )
                .unwrap(),
                vec![&mint_account, &account4_account, &owner_account],
            ),
            (
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
                    &account4_account,
                    &account2_account,
                    &multisig_account,
                    &account4_account,
                ],
            ),
        ],
        &[Check::success()],
    );

    // source-multisig-signer TransferChecked
    do_process_instructions_with_pre_instructions(
        Some(&setup_instructions),
        &[
            (
                initialize_multisig(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &multisig_key,
                    &[&account4_key],
                    1,
                )
                .unwrap(),
                vec![&multisig_account, &rent_sysvar, &account4_account],
            ),
            (
                initialize_account(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account4_key,
                    &mint_key,
                    &multisig_key,
                )
                .unwrap(),
                vec![
                    &account4_account,
                    &mint_account,
                    &multisig_account,
                    &rent_sysvar,
                ],
            ),
            (
                mint_to(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &mint_key,
                    &account4_key,
                    &owner_key,
                    &[],
                    1000,
                )
                .unwrap(),
                vec![&mint_account, &account4_account, &owner_account],
            ),
            (
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
                    &account4_account,
                    &mint_account,
                    &account2_account,
                    &multisig_account,
                    &account4_account,
                ],
            ),
        ],
        &[Check::success()],
    );
}

#[test]
fn test_transfer() {
    let program_id = TARGET_TOKEN_PROGRAM_ID;
    let account_key = Pubkey::new_unique();
    let account_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let account2_key = Pubkey::new_unique();
    let account2_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let account3_key = Pubkey::new_unique();
    let account3_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let delegate_key = Pubkey::new_unique();
    let delegate_account = SolanaAccount::default();
    let mismatch_key = Pubkey::new_unique();
    let mismatch_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &program_id,
    );
    let owner_key = Pubkey::new_unique();
    let owner_account = SolanaAccount::default();
    let owner2_key = Pubkey::new_unique();
    let owner2_account = SolanaAccount::default();
    let mint_key = Pubkey::new_unique();
    let mint_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let mint2_key = Pubkey::new_unique();
    let mint2_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
    let rent_sysvar = rent_sysvar();

    let setup_instructions = vec![
        // create mint
        (
            initialize_mint(
                &INSTRUCTION_TOKEN_PROGRAM_ID,
                &mint_key,
                &owner_key,
                None,
                2,
            )
            .unwrap(),
            vec![&mint_account, &rent_sysvar],
        ),
        // create account
        (
            initialize_account(
                &INSTRUCTION_TOKEN_PROGRAM_ID,
                &account_key,
                &mint_key,
                &owner_key,
            )
            .unwrap(),
            vec![
                &account_account,
                &mint_account,
                &owner_account,
                &rent_sysvar,
            ],
        ),
        // create another account
        (
            initialize_account(
                &INSTRUCTION_TOKEN_PROGRAM_ID,
                &account2_key,
                &mint_key,
                &owner_key,
            )
            .unwrap(),
            vec![
                &account2_account,
                &mint_account,
                &owner_account,
                &rent_sysvar,
            ],
        ),
        // create another account
        (
            initialize_account(
                &INSTRUCTION_TOKEN_PROGRAM_ID,
                &account3_key,
                &mint_key,
                &owner_key,
            )
            .unwrap(),
            vec![
                &account3_account,
                &mint_account,
                &owner_account,
                &rent_sysvar,
            ],
        ),
        // mint to account
        (
            mint_to(
                &INSTRUCTION_TOKEN_PROGRAM_ID,
                &mint_key,
                &account_key,
                &owner_key,
                &[],
                1000,
            )
            .unwrap(),
            vec![&mint_account, &account_account, &owner_account],
        ),
        // create a second mint
        (
            initialize_mint(
                &INSTRUCTION_TOKEN_PROGRAM_ID,
                &mint2_key,
                &owner_key,
                None,
                2,
            )
            .unwrap(),
            vec![&mint2_account, &rent_sysvar],
        ),
        // create mismatch account using mint2
        (
            initialize_account(
                &INSTRUCTION_TOKEN_PROGRAM_ID,
                &mismatch_key,
                &mint2_key,
                &owner_key,
            )
            .unwrap(),
            vec![
                &mismatch_account,
                &mint2_account,
                &owner_account,
                &rent_sysvar,
            ],
        ),
    ];

    // missing signer
    let mut instruction = transfer(
        &INSTRUCTION_TOKEN_PROGRAM_ID,
        &account_key,
        &account2_key,
        &owner_key,
        &[],
        1000,
    )
    .unwrap();
    instruction.accounts[2].is_signer = false;
    do_process_instructions_with_pre_instructions(
        Some(&setup_instructions),
        &[(
            instruction,
            vec![&account_account, &account2_account, &owner_account],
        )],
        &[Check::err(ProgramError::MissingRequiredSignature)],
    );

    // mismatch mint
    do_process_instructions_with_pre_instructions(
        Some(&setup_instructions),
        &[(
            transfer(
                &INSTRUCTION_TOKEN_PROGRAM_ID,
                &account_key,
                &mismatch_key,
                &owner_key,
                &[],
                1000,
            )
            .unwrap(),
            vec![&account_account, &mismatch_account, &owner_account],
        )],
        &[Check::err(TokenError::MintMismatch.into())],
    );

    // missing owner
    do_process_instructions_with_pre_instructions(
        Some(&setup_instructions),
        &[(
            transfer(
                &INSTRUCTION_TOKEN_PROGRAM_ID,
                &account_key,
                &account2_key,
                &owner2_key,
                &[],
                1000,
            )
            .unwrap(),
            vec![&account_account, &account2_account, &owner2_account],
        )],
        &[Check::err(TokenError::OwnerMismatch.into())],
    );

    // account not owned by program
    let not_program_key = Pubkey::new_unique();
    let mut account = Account::unpack_unchecked(&account_account.data).unwrap();
    account.state = AccountState::Initialized;
    account.mint = mint_key;
    account.owner = owner_key;
    let mut not_program_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &not_program_key,
    );
    Account::pack(account, &mut not_program_account.data).unwrap();

    do_process_instructions_with_pre_instructions(
        Some(&setup_instructions),
        &[(
            transfer(
                &INSTRUCTION_TOKEN_PROGRAM_ID,
                &not_program_key,
                &account2_key,
                &owner_key,
                &[],
                0,
            )
            .unwrap(),
            vec![&not_program_account, &account2_account, &owner2_account],
        )],
        &[Check::err(ProgramError::IncorrectProgramId)],
    );

    // account 2 not owned by program
    let mut account2 = Account::unpack_unchecked(&account_account.data).unwrap();
    account2.state = AccountState::Initialized;
    account2.mint = mint_key;
    account2.owner = owner_key;
    let mut not_program_account2 = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &not_program_key,
    );
    Account::pack(account2, &mut not_program_account2.data).unwrap();

    do_process_instructions_with_pre_instructions(
        Some(&setup_instructions),
        &[(
            transfer(
                &INSTRUCTION_TOKEN_PROGRAM_ID,
                &account_key,
                &not_program_key,
                &owner_key,
                &[],
                0,
            )
            .unwrap(),
            vec![&account_account, &not_program_account2, &owner2_account],
        )],
        &[Check::err(ProgramError::IncorrectProgramId)],
    );

    // transfer
    do_process_instructions_with_pre_instructions(
        Some(&setup_instructions),
        &[(
            transfer(
                &INSTRUCTION_TOKEN_PROGRAM_ID,
                &account_key,
                &account2_key,
                &owner_key,
                &[],
                1000,
            )
            .unwrap(),
            vec![&account_account, &account2_account, &owner_account],
        )],
        &[Check::success()],
    );

    // insufficient funds
    do_process_instructions_with_pre_instructions(
        Some(&setup_instructions),
        &[(
            transfer(
                &INSTRUCTION_TOKEN_PROGRAM_ID,
                &account_key,
                &account2_key,
                &owner_key,
                &[],
                1001,
            )
            .unwrap(),
            vec![&account_account, &account2_account, &owner_account],
        )],
        &[Check::err(TokenError::InsufficientFunds.into())],
    );

    // transfer half back
    do_process_instructions_with_pre_instructions(
        Some(&setup_instructions),
        &[
            (
                transfer(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account_key,
                    &account2_key,
                    &owner_key,
                    &[],
                    1000,
                )
                .unwrap(),
                vec![&account_account, &account2_account, &owner_account],
            ),
            (
                transfer(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account2_key,
                    &account_key,
                    &owner_key,
                    &[],
                    500,
                )
                .unwrap(),
                vec![&account2_account, &account_account, &owner_account],
            ),
        ],
        &[Check::success()],
    );

    // incorrect decimals
    do_process_instructions_with_pre_instructions(
        Some(&setup_instructions),
        &[
            (
                transfer(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account_key,
                    &account2_key,
                    &owner_key,
                    &[],
                    1000,
                )
                .unwrap(),
                vec![&account_account, &account2_account, &owner_account],
            ),
            (
                transfer_checked(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account2_key,
                    &mint_key,
                    &account_key,
                    &owner_key,
                    &[],
                    1,
                    10, // <-- incorrect decimals
                )
                .unwrap(),
                vec![
                    &account2_account,
                    &mint_account,
                    &account_account,
                    &owner_account,
                ],
            ),
        ],
        &[Check::err(TokenError::MintDecimalsMismatch.into())],
    );

    // incorrect mint
    do_process_instructions_with_pre_instructions(
        Some(&setup_instructions),
        &[
            (
                transfer(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account_key,
                    &account2_key,
                    &owner_key,
                    &[],
                    1000,
                )
                .unwrap(),
                vec![&account_account, &account2_account, &owner_account],
            ),
            (
                transfer_checked(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account2_key,
                    &account3_key, // <-- incorrect mint
                    &account_key,
                    &owner_key,
                    &[],
                    1,
                    2,
                )
                .unwrap(),
                vec![
                    &account2_account,
                    &account3_account, // <-- incorrect mint
                    &account_account,
                    &owner_account,
                ],
            ),
        ],
        &[Check::err(TokenError::MintMismatch.into())],
    );

    // transfer rest with explicit decimals
    do_process_instructions_with_pre_instructions(
        Some(&setup_instructions),
        &[
            (
                transfer(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account_key,
                    &account2_key,
                    &owner_key,
                    &[],
                    1000,
                )
                .unwrap(),
                vec![&account_account, &account2_account, &owner_account],
            ),
            (
                transfer(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account2_key,
                    &account_key,
                    &owner_key,
                    &[],
                    500,
                )
                .unwrap(),
                vec![&account2_account, &account_account, &owner_account],
            ),
            (
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
                    &account2_account,
                    &mint_account,
                    &account_account,
                    &owner_account,
                ],
            ),
        ],
        &[Check::success()],
    );
    /* TODO: seems to be done already
    // insufficient funds
    assert_eq!(
        Err(TokenError::InsufficientFunds.into()),
        do_process_instruction(
            transfer(
                &TOKEN_PROGRAM_ID,
                &account2_key,
                &account_key,
                &owner_key,
                &[],
                1
            )
            .unwrap(),
            vec![
                &mut account2_account,
                &mut account_account,
                &mut owner_account,
            ],
        )
    );
    */

    // approve delegate
    do_process_instructions_with_pre_instructions(
        Some(&setup_instructions),
        &[(
            approve(
                &INSTRUCTION_TOKEN_PROGRAM_ID,
                &account_key,
                &delegate_key,
                &owner_key,
                &[],
                100,
            )
            .unwrap(),
            vec![&account_account, &delegate_account, &owner_account],
        )],
        &[Check::success()],
    );

    // not a delegate of source account
    do_process_instructions_with_pre_instructions(
        Some(&setup_instructions),
        &[
            (
                approve(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account_key,
                    &delegate_key,
                    &owner_key,
                    &[],
                    100,
                )
                .unwrap(),
                vec![&account_account, &delegate_account, &owner_account],
            ),
            (
                transfer(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account_key,
                    &account2_key,
                    &owner2_key, // <-- incorrect owner or delegate
                    &[],
                    1,
                )
                .unwrap(),
                vec![&account_account, &account2_account, &owner2_account],
            ),
        ],
        &[Check::err(TokenError::OwnerMismatch.into())],
    );

    // insufficient funds approved via delegate
    do_process_instructions_with_pre_instructions(
        Some(&setup_instructions),
        &[
            (
                approve(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account_key,
                    &delegate_key,
                    &owner_key,
                    &[],
                    100,
                )
                .unwrap(),
                vec![&account_account, &delegate_account, &owner_account],
            ),
            (
                transfer(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account_key,
                    &account2_key,
                    &delegate_key,
                    &[],
                    101,
                )
                .unwrap(),
                vec![&account_account, &account2_account, &delegate_account],
            ),
        ],
        &[Check::err(TokenError::InsufficientFunds.into())],
    );

    // transfer via delegate
    do_process_instructions_with_pre_instructions(
        Some(&setup_instructions),
        &[
            (
                approve(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account_key,
                    &delegate_key,
                    &owner_key,
                    &[],
                    100,
                )
                .unwrap(),
                vec![&account_account, &delegate_account, &owner_account],
            ),
            (
                transfer(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account_key,
                    &account2_key,
                    &delegate_key,
                    &[],
                    100,
                )
                .unwrap(),
                vec![&account_account, &account2_account, &delegate_account],
            ),
        ],
        &[Check::success()],
    );

    // insufficient funds approved via delegate
    do_process_instructions_with_pre_instructions(
        Some(&setup_instructions),
        &[
            (
                approve(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account_key,
                    &delegate_key,
                    &owner_key,
                    &[],
                    100,
                )
                .unwrap(),
                vec![&account_account, &delegate_account, &owner_account],
            ),
            (
                transfer(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account_key,
                    &account2_key,
                    &delegate_key,
                    &[],
                    100,
                )
                .unwrap(),
                vec![&account_account, &account2_account, &delegate_account],
            ),
            (
                transfer(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account_key,
                    &account2_key,
                    &delegate_key,
                    &[],
                    1,
                )
                .unwrap(),
                vec![&account_account, &account2_account, &delegate_account],
            ),
        ],
        &[Check::err(TokenError::OwnerMismatch.into())],
    );

    // transfer rest
    do_process_instructions_with_pre_instructions(
        Some(&setup_instructions),
        &[
            (
                approve(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account_key,
                    &delegate_key,
                    &owner_key,
                    &[],
                    100,
                )
                .unwrap(),
                vec![&account_account, &delegate_account, &owner_account],
            ),
            (
                transfer(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account_key,
                    &account2_key,
                    &delegate_key,
                    &[],
                    100,
                )
                .unwrap(),
                vec![&account_account, &account2_account, &delegate_account],
            ),
            (
                transfer(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account_key,
                    &account2_key,
                    &owner_key,
                    &[],
                    900,
                )
                .unwrap(),
                vec![&account_account, &account2_account, &owner_account],
            ),
        ],
        &[Check::success()],
    );

    // approve delegate
    do_process_instructions_with_pre_instructions(
        Some(&setup_instructions),
        &[
            (
                approve(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account_key,
                    &delegate_key,
                    &owner_key,
                    &[],
                    100,
                )
                .unwrap(),
                vec![&account_account, &delegate_account, &owner_account],
            ),
            (
                transfer(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account_key,
                    &account2_key,
                    &delegate_key,
                    &[],
                    100,
                )
                .unwrap(),
                vec![&account_account, &account2_account, &delegate_account],
            ),
            (
                transfer(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account_key,
                    &account2_key,
                    &owner_key,
                    &[],
                    900,
                )
                .unwrap(),
                vec![&account_account, &account2_account, &owner_account],
            ),
            (
                approve(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account_key,
                    &delegate_key,
                    &owner_key,
                    &[],
                    100,
                )
                .unwrap(),
                vec![&account_account, &delegate_account, &owner_account],
            ),
        ],
        &[Check::success()],
    );

    // insufficient funds in source account via delegate
    do_process_instructions_with_pre_instructions(
        Some(&setup_instructions),
        &[
            (
                approve(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account_key,
                    &delegate_key,
                    &owner_key,
                    &[],
                    100,
                )
                .unwrap(),
                vec![&account_account, &delegate_account, &owner_account],
            ),
            (
                transfer(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account_key,
                    &account2_key,
                    &delegate_key,
                    &[],
                    100,
                )
                .unwrap(),
                vec![&account_account, &account2_account, &delegate_account],
            ),
            (
                transfer(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account_key,
                    &account2_key,
                    &owner_key,
                    &[],
                    900,
                )
                .unwrap(),
                vec![&account_account, &account2_account, &owner_account],
            ),
            (
                approve(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account_key,
                    &delegate_key,
                    &owner_key,
                    &[],
                    100,
                )
                .unwrap(),
                vec![&account_account, &delegate_account, &owner_account],
            ),
            (
                transfer(
                    &INSTRUCTION_TOKEN_PROGRAM_ID,
                    &account_key,
                    &account2_key,
                    &delegate_key,
                    &[],
                    100,
                )
                .unwrap(),
                vec![&account_account, &account2_account, &delegate_account],
            ),
        ],
        &[Check::err(TokenError::InsufficientFunds.into())],
    );
}
/*
    #[test]
    fn test_self_transfer() {
        let program_id = crate::id();
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
            initialize_mint(&TOKEN_PROGRAM_ID, &mint_key, &owner_key, None, 2).unwrap(),
            vec![&mut mint_account, &mut rent_sysvar],
        )
        .unwrap();

        // create account
        do_process_instruction(
            initialize_account(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &owner_key).unwrap(),
            vec![
                &mut account_account,
                &mut mint_account,
                &mut owner_account,
                &mut rent_sysvar,
            ],
        )
        .unwrap();

        // create another account
        do_process_instruction(
            initialize_account(&TOKEN_PROGRAM_ID, &account2_key, &mint_key, &owner_key).unwrap(),
            vec![
                &mut account2_account,
                &mut mint_account,
                &mut owner_account,
                &mut rent_sysvar,
            ],
        )
        .unwrap();

        // create another account
        do_process_instruction(
            initialize_account(&TOKEN_PROGRAM_ID, &account3_key, &mint_key, &owner_key).unwrap(),
            vec![
                &mut account3_account,
                &mut mint_account,
                &mut owner_account,
                &mut rent_sysvar,
            ],
        )
        .unwrap();

        // mint to account
        do_process_instruction(
            mint_to(&TOKEN_PROGRAM_ID, &mint_key, &account_key, &owner_key, &[], 1000).unwrap(),
            vec![&mut mint_account, &mut account_account, &mut owner_account],
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
            Processor::process(
                &instruction.program_id,
                &[
                    account_info.clone(),
                    account_info.clone(),
                    owner_info.clone(),
                ],
                &instruction.data,
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
            Processor::process(
                &instruction.program_id,
                &[
                    account_info.clone(),
                    mint_info.clone(),
                    account_info.clone(),
                    owner_info.clone(),
                ],
                &instruction.data,
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
            Processor::process(
                &instruction.program_id,
                &[
                    account_info.clone(),
                    account_info.clone(),
                    owner_no_sign_info.clone(),
                ],
                &instruction.data,
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
            Processor::process(
                &instruction.program_id,
                &[
                    account_info.clone(),
                    mint_info.clone(),
                    account_info.clone(),
                    owner_no_sign_info,
                ],
                &instruction.data,
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
            Processor::process(
                &instruction.program_id,
                &[
                    account_info.clone(),
                    account_info.clone(),
                    owner2_info.clone(),
                ],
                &instruction.data,
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
            Processor::process(
                &instruction.program_id,
                &[
                    account_info.clone(),
                    mint_info.clone(),
                    account_info.clone(),
                    owner2_info.clone(),
                ],
                &instruction.data,
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
            Processor::process(
                &instruction.program_id,
                &[
                    account_info.clone(),
                    account_info.clone(),
                    owner_info.clone(),
                ],
                &instruction.data,
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
            Processor::process(
                &instruction.program_id,
                &[
                    account_info.clone(),
                    mint_info.clone(),
                    account_info.clone(),
                    owner_info.clone(),
                ],
                &instruction.data,
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
            Processor::process(
                &instruction.program_id,
                &[
                    account_info.clone(),
                    mint_info.clone(),
                    account_info.clone(),
                    owner_info.clone(),
                ],
                &instruction.data,
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
            Processor::process(
                &instruction.program_id,
                &[
                    account_info.clone(),
                    account3_info.clone(), // <-- incorrect mint
                    account_info.clone(),
                    owner_info.clone(),
                ],
                &instruction.data,
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
        Processor::process(
            &instruction.program_id,
            &[
                account_info.clone(),
                delegate_info.clone(),
                owner_info.clone(),
            ],
            &instruction.data,
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
            Processor::process(
                &instruction.program_id,
                &[
                    account_info.clone(),
                    account_info.clone(),
                    delegate_info.clone(),
                ],
                &instruction.data,
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
            Processor::process(
                &instruction.program_id,
                &[
                    account_info.clone(),
                    mint_info.clone(),
                    account_info.clone(),
                    delegate_info.clone(),
                ],
                &instruction.data,
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
            Processor::process(
                &instruction.program_id,
                &[
                    account_info.clone(),
                    account_info.clone(),
                    delegate_info.clone(),
                ],
                &instruction.data,
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
            Processor::process(
                &instruction.program_id,
                &[
                    account_info.clone(),
                    mint_info.clone(),
                    account_info.clone(),
                    delegate_info.clone(),
                ],
                &instruction.data,
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
            Processor::process(
                &instruction.program_id,
                &[
                    account_info.clone(),
                    account_info.clone(),
                    owner_info.clone(),
                ],
                &instruction.data,
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
            Processor::process(
                &instruction.program_id,
                &[
                    account_info.clone(),
                    mint_info.clone(),
                    account_info.clone(),
                    owner_info.clone(),
                ],
                &instruction.data,
            )
        );
        // no balance change...
        let account = Account::unpack_unchecked(&account_info.try_borrow_data().unwrap()).unwrap();
        assert_eq!(account.amount, 1000);
    }

    #[test]
    fn test_mintable_token_with_zero_supply() {
        let program_id = crate::id();
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
        do_process_instruction(
            initialize_mint(&TOKEN_PROGRAM_ID, &mint_key, &owner_key, None, decimals).unwrap(),
            vec![&mut mint_account, &mut rent_sysvar],
        )
        .unwrap();
        let mint = Mint::unpack_unchecked(&mint_account.data).unwrap();
        assert_eq!(
            mint,
            Mint {
                mint_authority: COption::Some(owner_key),
                supply: 0,
                decimals,
                is_initialized: true,
                freeze_authority: COption::None,
            }
        );

        // create account
        do_process_instruction(
            initialize_account(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &owner_key).unwrap(),
            vec![
                &mut account_account,
                &mut mint_account,
                &mut owner_account,
                &mut rent_sysvar,
            ],
        )
        .unwrap();

        // mint to
        do_process_instruction(
            mint_to(&TOKEN_PROGRAM_ID, &mint_key, &account_key, &owner_key, &[], 42).unwrap(),
            vec![&mut mint_account, &mut account_account, &mut owner_account],
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
        )
        .unwrap();
        let _ = Mint::unpack(&mint_account.data).unwrap();
        let account = Account::unpack_unchecked(&account_account.data).unwrap();
        assert_eq!(account.amount, 84);
    }

    #[test]
    fn test_approve_dups() {
        let program_id = crate::id();
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
            initialize_mint(&TOKEN_PROGRAM_ID, &mint_key, &owner_key, None, 2).unwrap(),
            vec![mint_info.clone(), rent_info.clone()],
        )
        .unwrap();

        // create account
        do_process_instruction_dups(
            initialize_account(&TOKEN_PROGRAM_ID, &account1_key, &mint_key, &account1_key).unwrap(),
            vec![
                account1_info.clone(),
                mint_info.clone(),
                account1_info.clone(),
                rent_info.clone(),
            ],
        )
        .unwrap();

        // create another account
        do_process_instruction_dups(
            initialize_account(&TOKEN_PROGRAM_ID, &account2_key, &mint_key, &owner_key).unwrap(),
            vec![
                account2_info.clone(),
                mint_info.clone(),
                owner_info.clone(),
                rent_info.clone(),
            ],
        )
        .unwrap();

        // mint to account
        do_process_instruction_dups(
            mint_to(&TOKEN_PROGRAM_ID, &mint_key, &account1_key, &owner_key, &[], 1000).unwrap(),
            vec![mint_info.clone(), account1_info.clone(), owner_info.clone()],
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
        )
        .unwrap();

        // source-owner revoke
        do_process_instruction_dups(
            revoke(&TOKEN_PROGRAM_ID, &account1_key, &account1_key, &[]).unwrap(),
            vec![account1_info.clone(), account1_info.clone()],
        )
        .unwrap();

        // test source-multisig signer
        do_process_instruction_dups(
            initialize_multisig(&TOKEN_PROGRAM_ID, &multisig_key, &[&account3_key], 1).unwrap(),
            vec![
                multisig_info.clone(),
                rent_info.clone(),
                account3_info.clone(),
            ],
        )
        .unwrap();

        do_process_instruction_dups(
            initialize_account(&TOKEN_PROGRAM_ID, &account3_key, &mint_key, &multisig_key).unwrap(),
            vec![
                account3_info.clone(),
                mint_info.clone(),
                multisig_info.clone(),
                rent_info.clone(),
            ],
        )
        .unwrap();

        do_process_instruction_dups(
            mint_to(&TOKEN_PROGRAM_ID, &mint_key, &account3_key, &owner_key, &[], 1000).unwrap(),
            vec![mint_info.clone(), account3_info.clone(), owner_info.clone()],
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
        )
        .unwrap();

        // source-owner multisig-signer
        do_process_instruction_dups(
            revoke(&TOKEN_PROGRAM_ID, &account3_key, &multisig_key, &[&account3_key]).unwrap(),
            vec![
                account3_info.clone(),
                multisig_info.clone(),
                account3_info.clone(),
            ],
        )
        .unwrap();
    }

    #[test]
    fn test_approve() {
        let program_id = crate::id();
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
            initialize_mint(&TOKEN_PROGRAM_ID, &mint_key, &owner_key, None, 2).unwrap(),
            vec![&mut mint_account, &mut rent_sysvar],
        )
        .unwrap();

        // create account
        do_process_instruction(
            initialize_account(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &owner_key).unwrap(),
            vec![
                &mut account_account,
                &mut mint_account,
                &mut owner_account,
                &mut rent_sysvar,
            ],
        )
        .unwrap();

        // create another account
        do_process_instruction(
            initialize_account(&TOKEN_PROGRAM_ID, &account2_key, &mint_key, &owner_key).unwrap(),
            vec![
                &mut account2_account,
                &mut mint_account,
                &mut owner_account,
                &mut rent_sysvar,
            ],
        )
        .unwrap();

        // mint to account
        do_process_instruction(
            mint_to(&TOKEN_PROGRAM_ID, &mint_key, &account_key, &owner_key, &[], 1000).unwrap(),
            vec![&mut mint_account, &mut account_account, &mut owner_account],
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
        )
        .unwrap();

        // revoke delegate
        do_process_instruction(
            revoke(&TOKEN_PROGRAM_ID, &account_key, &owner_key, &[]).unwrap(),
            vec![&mut account_account, &mut owner_account],
        )
        .unwrap();
    }

    #[test]
    fn test_set_authority_dups() {
        let program_id = crate::id();
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
            initialize_mint(&TOKEN_PROGRAM_ID, &mint_key, &mint_key, Some(&mint_key), 2).unwrap(),
            vec![mint_info.clone(), rent_info.clone()],
        )
        .unwrap();

        // create account
        do_process_instruction_dups(
            initialize_account(&TOKEN_PROGRAM_ID, &account1_key, &mint_key, &account1_key).unwrap(),
            vec![
                account1_info.clone(),
                mint_info.clone(),
                account1_info.clone(),
                rent_info.clone(),
            ],
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
        )
        .unwrap();
    }

    #[test]
    fn test_set_authority() {
        let program_id = crate::id();
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
            initialize_mint(&TOKEN_PROGRAM_ID, &mint_key, &owner_key, None, 2).unwrap(),
            vec![&mut mint_account, &mut rent_sysvar],
        )
        .unwrap();

        // create mint with owner and freeze_authority
        do_process_instruction(
            initialize_mint(&TOKEN_PROGRAM_ID, &mint2_key, &owner_key, Some(&owner_key), 2).unwrap(),
            vec![&mut mint2_account, &mut rent_sysvar],
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
            )
        );

        // create account
        do_process_instruction(
            initialize_account(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &owner_key).unwrap(),
            vec![
                &mut account_account,
                &mut mint_account,
                &mut owner_account,
                &mut rent_sysvar,
            ],
        )
        .unwrap();

        // create another account
        do_process_instruction(
            initialize_account(&TOKEN_PROGRAM_ID, &account2_key, &mint2_key, &owner_key).unwrap(),
            vec![
                &mut account2_account,
                &mut mint2_account,
                &mut owner_account,
                &mut rent_sysvar,
            ],
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
            do_process_instruction(instruction, vec![&mut account_account, &mut owner_account,],)
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
            do_process_instruction(instruction, vec![&mut mint_account, &mut owner_account],)
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
            )
        );
    }

    #[test]
    fn test_mint_to_dups() {
        let program_id = crate::id();
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
            initialize_mint(&TOKEN_PROGRAM_ID, &mint_key, &mint_key, None, 2).unwrap(),
            vec![mint_info.clone(), rent_info.clone()],
        )
        .unwrap();

        // create account
        do_process_instruction_dups(
            initialize_account(&TOKEN_PROGRAM_ID, &account1_key, &mint_key, &owner_key).unwrap(),
            vec![
                account1_info.clone(),
                mint_info.clone(),
                owner_info.clone(),
                rent_info.clone(),
            ],
        )
        .unwrap();

        // mint_to when mint_authority is self
        do_process_instruction_dups(
            mint_to(&TOKEN_PROGRAM_ID, &mint_key, &account1_key, &mint_key, &[], 42).unwrap(),
            vec![mint_info.clone(), account1_info.clone(), mint_info.clone()],
        )
        .unwrap();

        // mint_to_checked when mint_authority is self
        do_process_instruction_dups(
            mint_to_checked(&TOKEN_PROGRAM_ID, &mint_key, &account1_key, &mint_key, &[], 42, 2).unwrap(),
            vec![mint_info.clone(), account1_info.clone(), mint_info.clone()],
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
        )
        .unwrap();
    }

    #[test]
    fn test_mint_to() {
        let program_id = crate::id();
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
            initialize_mint(&TOKEN_PROGRAM_ID, &mint_key, &owner_key, None, 2).unwrap(),
            vec![&mut mint_account, &mut rent_sysvar],
        )
        .unwrap();

        // create account
        do_process_instruction(
            initialize_account(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &owner_key).unwrap(),
            vec![
                &mut account_account,
                &mut mint_account,
                &mut owner_account,
                &mut rent_sysvar,
            ],
        )
        .unwrap();

        // create another account
        do_process_instruction(
            initialize_account(&TOKEN_PROGRAM_ID, &account2_key, &mint_key, &owner_key).unwrap(),
            vec![
                &mut account2_account,
                &mut mint_account,
                &mut owner_account,
                &mut rent_sysvar,
            ],
        )
        .unwrap();

        // create another account
        do_process_instruction(
            initialize_account(&TOKEN_PROGRAM_ID, &account3_key, &mint_key, &owner_key).unwrap(),
            vec![
                &mut account3_account,
                &mut mint_account,
                &mut owner_account,
                &mut rent_sysvar,
            ],
        )
        .unwrap();

        // create mismatch account
        do_process_instruction(
            initialize_account(&TOKEN_PROGRAM_ID, &mismatch_key, &mint_key, &owner_key).unwrap(),
            vec![
                &mut mismatch_account,
                &mut mint_account,
                &mut owner_account,
                &mut rent_sysvar,
            ],
        )
        .unwrap();
        let mut account = Account::unpack_unchecked(&mismatch_account.data).unwrap();
        account.mint = mint2_key;
        Account::pack(account, &mut mismatch_account.data).unwrap();

        // mint to
        do_process_instruction(
            mint_to(&TOKEN_PROGRAM_ID, &mint_key, &account_key, &owner_key, &[], 42).unwrap(),
            vec![&mut mint_account, &mut account_account, &mut owner_account],
        )
        .unwrap();

        let mint = Mint::unpack_unchecked(&mint_account.data).unwrap();
        assert_eq!(mint.supply, 42);
        let account = Account::unpack_unchecked(&account_account.data).unwrap();
        assert_eq!(account.amount, 42);

        // mint to another account to test supply accumulation
        do_process_instruction(
            mint_to(&TOKEN_PROGRAM_ID, &mint_key, &account2_key, &owner_key, &[], 42).unwrap(),
            vec![&mut mint_account, &mut account2_account, &mut owner_account],
        )
        .unwrap();

        let mint = Mint::unpack_unchecked(&mint_account.data).unwrap();
        assert_eq!(mint.supply, 84);
        let account = Account::unpack_unchecked(&account2_account.data).unwrap();
        assert_eq!(account.amount, 42);

        // missing signer
        let mut instruction =
            mint_to(&TOKEN_PROGRAM_ID, &mint_key, &account2_key, &owner_key, &[], 42).unwrap();
        instruction.accounts[2].is_signer = false;
        assert_eq!(
            Err(ProgramError::MissingRequiredSignature),
            do_process_instruction(
                instruction,
                vec![&mut mint_account, &mut account2_account, &mut owner_account],
            )
        );

        // mismatch account
        assert_eq!(
            Err(TokenError::MintMismatch.into()),
            do_process_instruction(
                mint_to(&TOKEN_PROGRAM_ID, &mint_key, &mismatch_key, &owner_key, &[], 42).unwrap(),
                vec![&mut mint_account, &mut mismatch_account, &mut owner_account],
            )
        );

        // missing owner
        assert_eq!(
            Err(TokenError::OwnerMismatch.into()),
            do_process_instruction(
                mint_to(&TOKEN_PROGRAM_ID, &mint_key, &account2_key, &owner2_key, &[], 42).unwrap(),
                vec![
                    &mut mint_account,
                    &mut account2_account,
                    &mut owner2_account,
                ],
            )
        );

        // mint not owned by program
        let not_program_id = Pubkey::new_unique();
        mint_account.owner = not_program_id;
        assert_eq!(
            Err(ProgramError::IncorrectProgramId),
            do_process_instruction(
                mint_to(&TOKEN_PROGRAM_ID, &mint_key, &account_key, &owner_key, &[], 0).unwrap(),
                vec![&mut mint_account, &mut account_account, &mut owner_account],
            )
        );
        mint_account.owner = program_id;

        // account not owned by program
        let not_program_id = Pubkey::new_unique();
        account_account.owner = not_program_id;
        assert_eq!(
            Err(ProgramError::IncorrectProgramId),
            do_process_instruction(
                mint_to(&TOKEN_PROGRAM_ID, &mint_key, &account_key, &owner_key, &[], 0).unwrap(),
                vec![&mut mint_account, &mut account_account, &mut owner_account],
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
        )
        .unwrap();
        assert_eq!(
            Err(TokenError::FixedSupply.into()),
            do_process_instruction(
                mint_to(&TOKEN_PROGRAM_ID, &mint_key, &account2_key, &owner_key, &[], 42).unwrap(),
                vec![&mut mint_account, &mut account2_account, &mut owner_account],
            )
        );
    }

    #[test]
    fn test_burn_dups() {
        let program_id = crate::id();
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
            initialize_mint(&TOKEN_PROGRAM_ID, &mint_key, &owner_key, None, 2).unwrap(),
            vec![mint_info.clone(), rent_info.clone()],
        )
        .unwrap();

        // create account
        do_process_instruction_dups(
            initialize_account(&TOKEN_PROGRAM_ID, &account1_key, &mint_key, &account1_key).unwrap(),
            vec![
                account1_info.clone(),
                mint_info.clone(),
                account1_info.clone(),
                rent_info.clone(),
            ],
        )
        .unwrap();

        // mint to account
        do_process_instruction_dups(
            mint_to(&TOKEN_PROGRAM_ID, &mint_key, &account1_key, &owner_key, &[], 1000).unwrap(),
            vec![mint_info.clone(), account1_info.clone(), owner_info.clone()],
        )
        .unwrap();

        // source-owner burn
        do_process_instruction_dups(
            burn(
                &program_id,
                &mint_key,
                &account1_key,
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
        )
        .unwrap();

        // mint-owner burn
        do_process_instruction_dups(
            mint_to(&TOKEN_PROGRAM_ID, &mint_key, &account1_key, &owner_key, &[], 1000).unwrap(),
            vec![mint_info.clone(), account1_info.clone(), owner_info.clone()],
        )
        .unwrap();
        let mut account = Account::unpack_unchecked(&account1_info.data.borrow()).unwrap();
        account.owner = mint_key;
        Account::pack(account, &mut account1_info.data.borrow_mut()).unwrap();
        do_process_instruction_dups(
            burn(&TOKEN_PROGRAM_ID, &account1_key, &mint_key, &mint_key, &[], 500).unwrap(),
            vec![account1_info.clone(), mint_info.clone(), mint_info.clone()],
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
        )
        .unwrap();

        // source-delegate burn
        do_process_instruction_dups(
            mint_to(&TOKEN_PROGRAM_ID, &mint_key, &account1_key, &owner_key, &[], 1000).unwrap(),
            vec![mint_info.clone(), account1_info.clone(), owner_info.clone()],
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
        )
        .unwrap();

        // mint-delegate burn
        do_process_instruction_dups(
            mint_to(&TOKEN_PROGRAM_ID, &mint_key, &account1_key, &owner_key, &[], 1000).unwrap(),
            vec![mint_info.clone(), account1_info.clone(), owner_info.clone()],
        )
        .unwrap();
        let mut account = Account::unpack_unchecked(&account1_info.data.borrow()).unwrap();
        account.delegated_amount = 1000;
        account.delegate = COption::Some(mint_key);
        account.owner = owner_key;
        Account::pack(account, &mut account1_info.data.borrow_mut()).unwrap();
        do_process_instruction_dups(
            burn(&TOKEN_PROGRAM_ID, &account1_key, &mint_key, &mint_key, &[], 500).unwrap(),
            vec![account1_info.clone(), mint_info.clone(), mint_info.clone()],
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
        )
        .unwrap();
    }

    #[test]
    fn test_burn() {
        let program_id = crate::id();
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
            initialize_mint(&TOKEN_PROGRAM_ID, &mint_key, &owner_key, None, 2).unwrap(),
            vec![&mut mint_account, &mut rent_sysvar],
        )
        .unwrap();

        // create account
        do_process_instruction(
            initialize_account(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &owner_key).unwrap(),
            vec![
                &mut account_account,
                &mut mint_account,
                &mut owner_account,
                &mut rent_sysvar,
            ],
        )
        .unwrap();

        // create another account
        do_process_instruction(
            initialize_account(&TOKEN_PROGRAM_ID, &account2_key, &mint_key, &owner_key).unwrap(),
            vec![
                &mut account2_account,
                &mut mint_account,
                &mut owner_account,
                &mut rent_sysvar,
            ],
        )
        .unwrap();

        // create another account
        do_process_instruction(
            initialize_account(&TOKEN_PROGRAM_ID, &account3_key, &mint_key, &owner_key).unwrap(),
            vec![
                &mut account3_account,
                &mut mint_account,
                &mut owner_account,
                &mut rent_sysvar,
            ],
        )
        .unwrap();

        // create mismatch account
        do_process_instruction(
            initialize_account(&TOKEN_PROGRAM_ID, &mismatch_key, &mint_key, &owner_key).unwrap(),
            vec![
                &mut mismatch_account,
                &mut mint_account,
                &mut owner_account,
                &mut rent_sysvar,
            ],
        )
        .unwrap();

        // mint to account
        do_process_instruction(
            mint_to(&TOKEN_PROGRAM_ID, &mint_key, &account_key, &owner_key, &[], 1000).unwrap(),
            vec![&mut mint_account, &mut account_account, &mut owner_account],
        )
        .unwrap();

        // mint to mismatch account and change mint key
        do_process_instruction(
            mint_to(&TOKEN_PROGRAM_ID, &mint_key, &mismatch_key, &owner_key, &[], 1000).unwrap(),
            vec![&mut mint_account, &mut mismatch_account, &mut owner_account],
        )
        .unwrap();
        let mut account = Account::unpack_unchecked(&mismatch_account.data).unwrap();
        account.mint = mint2_key;
        Account::pack(account, &mut mismatch_account.data).unwrap();

        // missing signer
        let mut instruction =
            burn(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &delegate_key, &[], 42).unwrap();
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
            )
        );

        // missing owner
        assert_eq!(
            Err(TokenError::OwnerMismatch.into()),
            do_process_instruction(
                burn(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &owner2_key, &[], 42).unwrap(),
                vec![&mut account_account, &mut mint_account, &mut owner2_account],
            )
        );

        // account not owned by program
        let not_program_id = Pubkey::new_unique();
        account_account.owner = not_program_id;
        assert_eq!(
            Err(ProgramError::IncorrectProgramId),
            do_process_instruction(
                burn(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &owner_key, &[], 0).unwrap(),
                vec![&mut account_account, &mut mint_account, &mut owner_account],
            )
        );
        account_account.owner = program_id;

        // mint not owned by program
        let not_program_id = Pubkey::new_unique();
        mint_account.owner = not_program_id;
        assert_eq!(
            Err(ProgramError::IncorrectProgramId),
            do_process_instruction(
                burn(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &owner_key, &[], 0).unwrap(),
                vec![&mut account_account, &mut mint_account, &mut owner_account],
            )
        );
        mint_account.owner = program_id;

        // mint mismatch
        assert_eq!(
            Err(TokenError::MintMismatch.into()),
            do_process_instruction(
                burn(&TOKEN_PROGRAM_ID, &mismatch_key, &mint_key, &owner_key, &[], 42).unwrap(),
                vec![&mut mismatch_account, &mut mint_account, &mut owner_account],
            )
        );

        // burn
        do_process_instruction(
            burn(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &owner_key, &[], 21).unwrap(),
            vec![&mut account_account, &mut mint_account, &mut owner_account],
        )
        .unwrap();

        // burn_checked, with incorrect decimals
        assert_eq!(
            Err(TokenError::MintDecimalsMismatch.into()),
            do_process_instruction(
                burn_checked(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &owner_key, &[], 21, 3).unwrap(),
                vec![&mut account_account, &mut mint_account, &mut owner_account],
            )
        );

        // burn_checked
        do_process_instruction(
            burn_checked(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &owner_key, &[], 21, 2).unwrap(),
            vec![&mut account_account, &mut mint_account, &mut owner_account],
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
            )
        );

        // insufficient funds approved via delegate
        assert_eq!(
            Err(TokenError::InsufficientFunds.into()),
            do_process_instruction(
                burn(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &delegate_key, &[], 85).unwrap(),
                vec![
                    &mut account_account,
                    &mut mint_account,
                    &mut delegate_account
                ],
            )
        );

        // burn via delegate
        do_process_instruction(
            burn(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &delegate_key, &[], 84).unwrap(),
            vec![
                &mut account_account,
                &mut mint_account,
                &mut delegate_account,
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
                burn(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &delegate_key, &[], 1).unwrap(),
                vec![
                    &mut account_account,
                    &mut mint_account,
                    &mut delegate_account
                ],
            )
        );
    }

    #[test]
    fn test_burn_and_close_system_and_incinerator_tokens() {
        let program_id = crate::id();
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
            initialize_mint2(&TOKEN_PROGRAM_ID, &mint_key, &owner_key, None, 2).unwrap(),
            vec![&mut mint_account],
        )
        .unwrap();

        // create account
        do_process_instruction(
            initialize_account3(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &owner_key).unwrap(),
            vec![&mut account_account, &mut mint_account],
        )
        .unwrap();

        // create incinerator- and system-owned accounts
        do_process_instruction(
            initialize_account3(
                &program_id,
                &incinerator_account_key,
                &mint_key,
                &solana_program::incinerator::id(),
            )
            .unwrap(),
            vec![&mut incinerator_account, &mut mint_account],
        )
        .unwrap();
        do_process_instruction(
            initialize_account3(
                &program_id,
                &system_account_key,
                &mint_key,
                &solana_program::system_program::id(),
            )
            .unwrap(),
            vec![&mut system_account, &mut mint_account],
        )
        .unwrap();

        // mint to account
        do_process_instruction(
            mint_to(&TOKEN_PROGRAM_ID, &mint_key, &account_key, &owner_key, &[], 1000).unwrap(),
            vec![&mut mint_account, &mut account_account, &mut owner_account],
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
        )
        .unwrap();

        // close with balance fails
        assert_eq!(
            Err(TokenError::NonNativeHasBalance.into()),
            do_process_instruction(
                close_account(
                    &program_id,
                    &incinerator_account_key,
                    &solana_program::incinerator::id(),
                    &owner_key,
                    &[]
                )
                .unwrap(),
                vec![
                    &mut incinerator_account,
                    &mut mock_incinerator_account,
                    &mut owner_account,
                ],
            )
        );
        assert_eq!(
            Err(TokenError::NonNativeHasBalance.into()),
            do_process_instruction(
                close_account(
                    &program_id,
                    &system_account_key,
                    &solana_program::incinerator::id(),
                    &owner_key,
                    &[]
                )
                .unwrap(),
                vec![
                    &mut system_account,
                    &mut mock_incinerator_account,
                    &mut owner_account,
                ],
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
            )
        );

        // closing succeeds with incinerator recipient
        do_process_instruction(
            close_account(
                &program_id,
                &incinerator_account_key,
                &solana_program::incinerator::id(),
                &owner_key,
                &[],
            )
            .unwrap(),
            vec![
                &mut incinerator_account,
                &mut mock_incinerator_account,
                &mut owner_account,
            ],
        )
        .unwrap();

        do_process_instruction(
            close_account(
                &program_id,
                &system_account_key,
                &solana_program::incinerator::id(),
                &owner_key,
                &[],
            )
            .unwrap(),
            vec![
                &mut system_account,
                &mut mock_incinerator_account,
                &mut owner_account,
            ],
        )
        .unwrap();
    }

    #[test]
    fn test_multisig() {
        let program_id = crate::id();
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
                initialize_multisig(&TOKEN_PROGRAM_ID, &multisig_key, &[&signer_keys[0]], 1).unwrap(),
                vec![
                    &mut multisig_account,
                    &mut rent_sysvar,
                    account_info_iter.next().unwrap(),
                ],
            )
        );

        multisig_account.lamports = multisig_minimum_balance();
        let mut multisig_account2 = multisig_account.clone();

        // single signer
        let account_info_iter = &mut signer_accounts.iter_mut();
        do_process_instruction(
            initialize_multisig(&TOKEN_PROGRAM_ID, &multisig_key, &[&signer_keys[0]], 1).unwrap(),
            vec![
                &mut multisig_account,
                &mut rent_sysvar,
                account_info_iter.next().unwrap(),
            ],
        )
        .unwrap();

        // single signer using `initialize_multisig2`
        let account_info_iter = &mut signer_accounts.iter_mut();
        do_process_instruction(
            initialize_multisig2(&TOKEN_PROGRAM_ID, &multisig_key, &[&signer_keys[0]], 1).unwrap(),
            vec![&mut multisig_account2, account_info_iter.next().unwrap()],
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
        )
        .unwrap();

        // create new mint with multisig owner
        do_process_instruction(
            initialize_mint(&TOKEN_PROGRAM_ID, &mint_key, &multisig_key, None, 2).unwrap(),
            vec![&mut mint_account, &mut rent_sysvar],
        )
        .unwrap();

        // create account with multisig owner
        do_process_instruction(
            initialize_account(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &multisig_key).unwrap(),
            vec![
                &mut account,
                &mut mint_account,
                &mut multisig_account,
                &mut rent_sysvar,
            ],
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
        )
        .unwrap();
        do_process_instruction(
            initialize_account(&TOKEN_PROGRAM_ID, &account3_key, &mint2_key, &owner_key).unwrap(),
            vec![
                &mut account3_account,
                &mut mint2_account,
                &mut owner_account,
                &mut rent_sysvar,
            ],
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
        )
        .unwrap();
    }

    #[test]
    fn test_validate_owner() {
        let program_id = crate::id();
        let owner_key = Pubkey::new_unique();
        let mut signer_keys = [Pubkey::default(); MAX_SIGNERS];
        for signer_key in signer_keys.iter_mut().take(MAX_SIGNERS) {
            *signer_key = Pubkey::new_unique();
        }
        let mut signer_lamports = 0;
        let mut signer_data = vec![];
        let mut signers = vec![
            AccountInfo::new(
                &owner_key,
                true,
                false,
                &mut signer_lamports,
                &mut signer_data,
                &program_id,
                false,
                Epoch::default(),
            );
            MAX_SIGNERS + 1
        ];
        for (signer, key) in signers.iter_mut().zip(&signer_keys) {
            signer.key = key;
        }
        let mut lamports = 0;
        let mut data = vec![0; Multisig::get_packed_len()];
        let mut multisig = Multisig::unpack_unchecked(&data).unwrap();
        multisig.m = MAX_SIGNERS as u8;
        multisig.n = MAX_SIGNERS as u8;
        multisig.signers = signer_keys;
        multisig.is_initialized = true;
        Multisig::pack(multisig, &mut data).unwrap();
        let owner_account_info = AccountInfo::new(
            &owner_key,
            false,
            false,
            &mut lamports,
            &mut data,
            &program_id,
            false,
            Epoch::default(),
        );

        // full 11 of 11
        Processor::validate_owner(&TOKEN_PROGRAM_ID, &owner_key, &owner_account_info, &signers).unwrap();

        // 1 of 11
        {
            let mut multisig =
                Multisig::unpack_unchecked(&owner_account_info.data.borrow()).unwrap();
            multisig.m = 1;
            Multisig::pack(multisig, &mut owner_account_info.data.borrow_mut()).unwrap();
        }
        Processor::validate_owner(&TOKEN_PROGRAM_ID, &owner_key, &owner_account_info, &signers).unwrap();

        // 2:1
        {
            let mut multisig =
                Multisig::unpack_unchecked(&owner_account_info.data.borrow()).unwrap();
            multisig.m = 2;
            multisig.n = 1;
            Multisig::pack(multisig, &mut owner_account_info.data.borrow_mut()).unwrap();
        }
        assert_eq!(
            Err(ProgramError::MissingRequiredSignature),
            Processor::validate_owner(&TOKEN_PROGRAM_ID, &owner_key, &owner_account_info, &signers)
        );

        // 0:11
        {
            let mut multisig =
                Multisig::unpack_unchecked(&owner_account_info.data.borrow()).unwrap();
            multisig.m = 0;
            multisig.n = 11;
            Multisig::pack(multisig, &mut owner_account_info.data.borrow_mut()).unwrap();
        }
        Processor::validate_owner(&TOKEN_PROGRAM_ID, &owner_key, &owner_account_info, &signers).unwrap();

        // 2:11 but 0 provided
        {
            let mut multisig =
                Multisig::unpack_unchecked(&owner_account_info.data.borrow()).unwrap();
            multisig.m = 2;
            multisig.n = 11;
            Multisig::pack(multisig, &mut owner_account_info.data.borrow_mut()).unwrap();
        }
        assert_eq!(
            Err(ProgramError::MissingRequiredSignature),
            Processor::validate_owner(&TOKEN_PROGRAM_ID, &owner_key, &owner_account_info, &[])
        );
        // 2:11 but 1 provided
        {
            let mut multisig =
                Multisig::unpack_unchecked(&owner_account_info.data.borrow()).unwrap();
            multisig.m = 2;
            multisig.n = 11;
            Multisig::pack(multisig, &mut owner_account_info.data.borrow_mut()).unwrap();
        }
        assert_eq!(
            Err(ProgramError::MissingRequiredSignature),
            Processor::validate_owner(&TOKEN_PROGRAM_ID, &owner_key, &owner_account_info, &signers[0..1])
        );

        // 2:11, 2 from middle provided
        {
            let mut multisig =
                Multisig::unpack_unchecked(&owner_account_info.data.borrow()).unwrap();
            multisig.m = 2;
            multisig.n = 11;
            Multisig::pack(multisig, &mut owner_account_info.data.borrow_mut()).unwrap();
        }
        Processor::validate_owner(&TOKEN_PROGRAM_ID, &owner_key, &owner_account_info, &signers[5..7])
            .unwrap();

        // 11:11, one is not a signer
        {
            let mut multisig =
                Multisig::unpack_unchecked(&owner_account_info.data.borrow()).unwrap();
            multisig.m = 11;
            multisig.n = 11;
            Multisig::pack(multisig, &mut owner_account_info.data.borrow_mut()).unwrap();
        }
        signers[5].is_signer = false;
        assert_eq!(
            Err(ProgramError::MissingRequiredSignature),
            Processor::validate_owner(&TOKEN_PROGRAM_ID, &owner_key, &owner_account_info, &signers)
        );
        signers[5].is_signer = true;

        // 11:11, single signer signs multiple times
        {
            let mut signer_lamports = 0;
            let mut signer_data = vec![];
            let signers = vec![
                AccountInfo::new(
                    &signer_keys[5],
                    true,
                    false,
                    &mut signer_lamports,
                    &mut signer_data,
                    &program_id,
                    false,
                    Epoch::default(),
                );
                MAX_SIGNERS + 1
            ];
            let mut multisig =
                Multisig::unpack_unchecked(&owner_account_info.data.borrow()).unwrap();
            multisig.m = 11;
            multisig.n = 11;
            Multisig::pack(multisig, &mut owner_account_info.data.borrow_mut()).unwrap();
            assert_eq!(
                Err(ProgramError::MissingRequiredSignature),
                Processor::validate_owner(&TOKEN_PROGRAM_ID, &owner_key, &owner_account_info, &signers)
            );
        }
    }

    #[test]
    fn test_owner_close_account_dups() {
        let program_id = crate::id();
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
            initialize_mint(&TOKEN_PROGRAM_ID, &mint_key, &owner_key, None, 2).unwrap(),
            vec![mint_info.clone(), rent_info.clone()],
        )
        .unwrap();

        let to_close_key = Pubkey::new_unique();
        let mut to_close_account = SolanaAccount::new(
            account_minimum_balance(),
            Account::get_packed_len(),
            &program_id,
        );
        let to_close_account_info: AccountInfo =
            (&to_close_key, true, &mut to_close_account).into();
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
            initialize_account(&TOKEN_PROGRAM_ID, &to_close_key, &mint_key, &to_close_key).unwrap(),
            vec![
                to_close_account_info.clone(),
                mint_info.clone(),
                to_close_account_info.clone(),
                rent_info.clone(),
            ],
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
        )
        .unwrap();
        assert_eq!(*to_close_account_info.data.borrow(), &[0u8; Account::LEN]);
    }

    #[test]
    fn test_close_authority_close_account_dups() {
        let program_id = crate::id();
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
            initialize_mint(&TOKEN_PROGRAM_ID, &mint_key, &owner_key, None, 2).unwrap(),
            vec![mint_info.clone(), rent_info.clone()],
        )
        .unwrap();

        let to_close_key = Pubkey::new_unique();
        let mut to_close_account = SolanaAccount::new(
            account_minimum_balance(),
            Account::get_packed_len(),
            &program_id,
        );
        let to_close_account_info: AccountInfo =
            (&to_close_key, true, &mut to_close_account).into();
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
            initialize_account(&TOKEN_PROGRAM_ID, &to_close_key, &mint_key, &to_close_key).unwrap(),
            vec![
                to_close_account_info.clone(),
                mint_info.clone(),
                to_close_account_info.clone(),
                rent_info.clone(),
            ],
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
        )
        .unwrap();
        assert_eq!(*to_close_account_info.data.borrow(), &[0u8; Account::LEN]);
    }

    #[test]
    fn test_close_account() {
        let program_id = crate::id();
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
                close_account(&TOKEN_PROGRAM_ID, &account_key, &account3_key, &owner2_key, &[]).unwrap(),
                vec![
                    &mut account_account,
                    &mut account3_account,
                    &mut owner2_account,
                ],
            )
        );

        // initialize and mint to non-native account
        do_process_instruction(
            initialize_mint(&TOKEN_PROGRAM_ID, &mint_key, &owner_key, None, 2).unwrap(),
            vec![&mut mint_account, &mut rent_sysvar],
        )
        .unwrap();
        do_process_instruction(
            initialize_account(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &owner_key).unwrap(),
            vec![
                &mut account_account,
                &mut mint_account,
                &mut owner_account,
                &mut rent_sysvar,
            ],
        )
        .unwrap();
        do_process_instruction(
            mint_to(&TOKEN_PROGRAM_ID, &mint_key, &account_key, &owner_key, &[], 42).unwrap(),
            vec![
                &mut mint_account,
                &mut account_account,
                &mut owner_account,
                &mut rent_sysvar,
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
                &crate::native_mint::id(),
                &owner_key,
            )
            .unwrap(),
            vec![
                &mut account2_account,
                &mut mint_account,
                &mut owner_account,
                &mut rent_sysvar,
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
                close_account(&TOKEN_PROGRAM_ID, &account_key, &account3_key, &owner_key, &[]).unwrap(),
                vec![
                    &mut account_account,
                    &mut account3_account,
                    &mut owner_account,
                ],
            )
        );
        assert_eq!(account_account.lamports, account_minimum_balance());

        // empty account
        do_process_instruction(
            burn(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &owner_key, &[], 42).unwrap(),
            vec![&mut account_account, &mut mint_account, &mut owner_account],
        )
        .unwrap();

        // wrong owner
        assert_eq!(
            Err(TokenError::OwnerMismatch.into()),
            do_process_instruction(
                close_account(&TOKEN_PROGRAM_ID, &account_key, &account3_key, &owner2_key, &[]).unwrap(),
                vec![
                    &mut account_account,
                    &mut account3_account,
                    &mut owner2_account,
                ],
            )
        );

        // close account
        do_process_instruction(
            close_account(&TOKEN_PROGRAM_ID, &account_key, &account3_key, &owner_key, &[]).unwrap(),
            vec![
                &mut account_account,
                &mut account3_account,
                &mut owner_account,
            ],
        )
        .unwrap();
        assert_eq!(account_account.lamports, 0);
        assert_eq!(account3_account.lamports, 2 * account_minimum_balance());
        let account = Account::unpack_unchecked(&account_account.data).unwrap();
        assert_eq!(account.amount, 0);

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
            initialize_account(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &owner_key).unwrap(),
            vec![
                &mut account_account,
                &mut mint_account,
                &mut owner_account,
                &mut rent_sysvar,
            ],
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
        )
        .unwrap();

        // account owner cannot authorize close if close_authority is set
        assert_eq!(
            Err(TokenError::OwnerMismatch.into()),
            do_process_instruction(
                close_account(&TOKEN_PROGRAM_ID, &account_key, &account3_key, &owner_key, &[]).unwrap(),
                vec![
                    &mut account_account,
                    &mut account3_account,
                    &mut owner_account,
                ],
            )
        );

        // close non-native account with close_authority
        do_process_instruction(
            close_account(&TOKEN_PROGRAM_ID, &account_key, &account3_key, &owner2_key, &[]).unwrap(),
            vec![
                &mut account_account,
                &mut account3_account,
                &mut owner2_account,
            ],
        )
        .unwrap();
        assert_eq!(account_account.lamports, 0);
        assert_eq!(account3_account.lamports, 2 * account_minimum_balance() + 2);
        let account = Account::unpack_unchecked(&account_account.data).unwrap();
        assert_eq!(account.amount, 0);

        // close native account
        do_process_instruction(
            close_account(&TOKEN_PROGRAM_ID, &account2_key, &account3_key, &owner_key, &[]).unwrap(),
            vec![
                &mut account2_account,
                &mut account3_account,
                &mut owner_account,
            ],
        )
        .unwrap();
        assert_eq!(account2_account.data, [0u8; Account::LEN]);
        assert_eq!(
            account3_account.lamports,
            3 * account_minimum_balance() + 2 + 42
        );
    }

    #[test]
    fn test_native_token() {
        let program_id = crate::id();
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
                &crate::native_mint::id(),
                &owner_key,
            )
            .unwrap(),
            vec![
                &mut account_account,
                &mut mint_account,
                &mut owner_account,
                &mut rent_sysvar,
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
                &crate::native_mint::id(),
                &owner_key,
            )
            .unwrap(),
            vec![
                &mut account2_account,
                &mut mint_account,
                &mut owner_account,
                &mut rent_sysvar,
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
                    &crate::native_mint::id(),
                    &account_key,
                    &owner_key,
                    &[],
                    42
                )
                .unwrap(),
                vec![&mut mint_account, &mut account_account, &mut owner_account],
            )
        );

        // burn unsupported
        let bogus_mint_key = Pubkey::new_unique();
        let mut bogus_mint_account =
            SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
        do_process_instruction(
            initialize_mint(&TOKEN_PROGRAM_ID, &bogus_mint_key, &owner_key, None, 2).unwrap(),
            vec![&mut bogus_mint_account, &mut rent_sysvar],
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
        )
        .unwrap();

        // close authority cleared
        let account = Account::unpack_unchecked(&account_account.data).unwrap();
        assert_eq!(account.close_authority, COption::None);

        // close native account
        do_process_instruction(
            close_account(&TOKEN_PROGRAM_ID, &account_key, &account3_key, &owner2_key, &[]).unwrap(),
            vec![
                &mut account_account,
                &mut account3_account,
                &mut owner2_account,
            ],
        )
        .unwrap();
        assert_eq!(account_account.lamports, 0);
        assert_eq!(account3_account.lamports, 2 * account_minimum_balance());
        assert_eq!(account_account.data, [0u8; Account::LEN]);
    }

    #[test]
    fn test_overflow() {
        let program_id = crate::id();
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
            initialize_mint(&TOKEN_PROGRAM_ID, &mint_key, &mint_owner_key, None, 2).unwrap(),
            vec![&mut mint_account, &mut rent_sysvar],
        )
        .unwrap();

        // create an account
        do_process_instruction(
            initialize_account(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &owner_key).unwrap(),
            vec![
                &mut account_account,
                &mut mint_account,
                &mut owner_account,
                &mut rent_sysvar,
            ],
        )
        .unwrap();

        // create another account
        do_process_instruction(
            initialize_account(&TOKEN_PROGRAM_ID, &account2_key, &mint_key, &owner2_key).unwrap(),
            vec![
                &mut account2_account,
                &mut mint_account,
                &mut owner2_account,
                &mut rent_sysvar,
            ],
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
            )
        );

        // burn some of the supply
        do_process_instruction(
            burn(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &owner_key, &[], 100).unwrap(),
            vec![&mut account_account, &mut mint_account, &mut owner_account],
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
        )
        .unwrap();
        let account = Account::unpack_unchecked(&account_account.data).unwrap();
        assert_eq!(account.amount, u64::MAX);

        // manipulate account balance to attempt overflow transfer
        let mut account = Account::unpack_unchecked(&account2_account.data).unwrap();
        account.amount = 1;
        Account::pack(account, &mut account2_account.data).unwrap();

        assert_eq!(
            Err(TokenError::Overflow.into()),
            do_process_instruction(
                transfer(
                    &program_id,
                    &account2_key,
                    &account_key,
                    &owner2_key,
                    &[],
                    1,
                )
                .unwrap(),
                vec![
                    &mut account2_account,
                    &mut account_account,
                    &mut owner2_account,
                ],
            )
        );
    }

    #[test]
    fn test_frozen() {
        let program_id = crate::id();
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
            initialize_mint(&TOKEN_PROGRAM_ID, &mint_key, &owner_key, None, 2).unwrap(),
            vec![&mut mint_account, &mut rent_sysvar],
        )
        .unwrap();

        // create account
        do_process_instruction(
            initialize_account(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &owner_key).unwrap(),
            vec![
                &mut account_account,
                &mut mint_account,
                &mut owner_account,
                &mut rent_sysvar,
            ],
        )
        .unwrap();

        // create another account
        do_process_instruction(
            initialize_account(&TOKEN_PROGRAM_ID, &account2_key, &mint_key, &owner_key).unwrap(),
            vec![
                &mut account2_account,
                &mut mint_account,
                &mut owner_account,
                &mut rent_sysvar,
            ],
        )
        .unwrap();

        // fund first account
        do_process_instruction(
            mint_to(&TOKEN_PROGRAM_ID, &mint_key, &account_key, &owner_key, &[], 1000).unwrap(),
            vec![&mut mint_account, &mut account_account, &mut owner_account],
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
                revoke(&TOKEN_PROGRAM_ID, &account_key, &owner_key, &[]).unwrap(),
                vec![&mut account_account, &mut owner_account],
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
            )
        );

        // no mint_to if destination account is frozen
        assert_eq!(
            Err(TokenError::AccountFrozen.into()),
            do_process_instruction(
                mint_to(&TOKEN_PROGRAM_ID, &mint_key, &account_key, &owner_key, &[], 100).unwrap(),
                vec![&mut mint_account, &mut account_account, &mut owner_account,],
            )
        );

        // no burn if account is frozen
        assert_eq!(
            Err(TokenError::AccountFrozen.into()),
            do_process_instruction(
                burn(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &owner_key, &[], 100).unwrap(),
                vec![&mut account_account, &mut mint_account, &mut owner_account],
            )
        );
    }

    #[test]
    fn test_freeze_thaw_dups() {
        let program_id = crate::id();
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
            initialize_mint(&TOKEN_PROGRAM_ID, &mint_key, &owner_key, Some(&account1_key), 2).unwrap(),
            vec![mint_info.clone(), rent_info.clone()],
        )
        .unwrap();

        // create account
        do_process_instruction_dups(
            initialize_account(&TOKEN_PROGRAM_ID, &account1_key, &mint_key, &account1_key).unwrap(),
            vec![
                account1_info.clone(),
                mint_info.clone(),
                account1_info.clone(),
                rent_info.clone(),
            ],
        )
        .unwrap();

        // freeze where mint freeze_authority is account
        do_process_instruction_dups(
            freeze_account(&TOKEN_PROGRAM_ID, &account1_key, &mint_key, &account1_key, &[]).unwrap(),
            vec![
                account1_info.clone(),
                mint_info.clone(),
                account1_info.clone(),
            ],
        )
        .unwrap();

        // thaw where mint freeze_authority is account
        let mut account = Account::unpack_unchecked(&account1_info.data.borrow()).unwrap();
        account.state = AccountState::Frozen;
        Account::pack(account, &mut account1_info.data.borrow_mut()).unwrap();
        do_process_instruction_dups(
            thaw_account(&TOKEN_PROGRAM_ID, &account1_key, &mint_key, &account1_key, &[]).unwrap(),
            vec![
                account1_info.clone(),
                mint_info.clone(),
                account1_info.clone(),
            ],
        )
        .unwrap();
    }

    #[test]
    fn test_freeze_account() {
        let program_id = crate::id();
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
            initialize_mint(&TOKEN_PROGRAM_ID, &mint_key, &owner_key, None, 2).unwrap(),
            vec![&mut mint_account, &mut rent_sysvar],
        )
        .unwrap();

        // create account
        do_process_instruction(
            initialize_account(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &account_owner_key).unwrap(),
            vec![
                &mut account_account,
                &mut mint_account,
                &mut account_owner_account,
                &mut rent_sysvar,
            ],
        )
        .unwrap();

        // mint to account
        do_process_instruction(
            mint_to(&TOKEN_PROGRAM_ID, &mint_key, &account_key, &owner_key, &[], 1000).unwrap(),
            vec![&mut mint_account, &mut account_account, &mut owner_account],
        )
        .unwrap();

        // mint cannot freeze
        assert_eq!(
            Err(TokenError::MintCannotFreeze.into()),
            do_process_instruction(
                freeze_account(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &owner_key, &[]).unwrap(),
                vec![&mut account_account, &mut mint_account, &mut owner_account],
            )
        );

        // missing freeze_authority
        let mut mint = Mint::unpack_unchecked(&mint_account.data).unwrap();
        mint.freeze_authority = COption::Some(owner_key);
        Mint::pack(mint, &mut mint_account.data).unwrap();
        assert_eq!(
            Err(TokenError::OwnerMismatch.into()),
            do_process_instruction(
                freeze_account(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &owner2_key, &[]).unwrap(),
                vec![&mut account_account, &mut mint_account, &mut owner2_account],
            )
        );

        // check explicit thaw
        assert_eq!(
            Err(TokenError::InvalidState.into()),
            do_process_instruction(
                thaw_account(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &owner2_key, &[]).unwrap(),
                vec![&mut account_account, &mut mint_account, &mut owner2_account],
            )
        );

        // freeze
        do_process_instruction(
            freeze_account(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &owner_key, &[]).unwrap(),
            vec![&mut account_account, &mut mint_account, &mut owner_account],
        )
        .unwrap();
        let account = Account::unpack_unchecked(&account_account.data).unwrap();
        assert_eq!(account.state, AccountState::Frozen);

        // check explicit freeze
        assert_eq!(
            Err(TokenError::InvalidState.into()),
            do_process_instruction(
                freeze_account(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &owner_key, &[]).unwrap(),
                vec![&mut account_account, &mut mint_account, &mut owner_account],
            )
        );

        // check thaw authority
        assert_eq!(
            Err(TokenError::OwnerMismatch.into()),
            do_process_instruction(
                thaw_account(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &owner2_key, &[]).unwrap(),
                vec![&mut account_account, &mut mint_account, &mut owner2_account],
            )
        );

        // thaw
        do_process_instruction(
            thaw_account(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &owner_key, &[]).unwrap(),
            vec![&mut account_account, &mut mint_account, &mut owner_account],
        )
        .unwrap();
        let account = Account::unpack_unchecked(&account_account.data).unwrap();
        assert_eq!(account.state, AccountState::Initialized);
    }

    #[test]
    fn test_initialize_account2_and_3() {
        let program_id = crate::id();
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
            initialize_mint(&TOKEN_PROGRAM_ID, &mint_key, &owner_key, None, 2).unwrap(),
            vec![&mut mint_account, &mut rent_sysvar],
        )
        .unwrap();

        do_process_instruction(
            initialize_account(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &owner_key).unwrap(),
            vec![
                &mut account_account,
                &mut mint_account,
                &mut owner_account,
                &mut rent_sysvar,
            ],
        )
        .unwrap();

        do_process_instruction(
            initialize_account2(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &owner_key).unwrap(),
            vec![&mut account2_account, &mut mint_account, &mut rent_sysvar],
        )
        .unwrap();

        assert_eq!(account_account, account2_account);

        do_process_instruction(
            initialize_account3(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &owner_key).unwrap(),
            vec![&mut account3_account, &mut mint_account],
        )
        .unwrap();

        assert_eq!(account_account, account3_account);
    }

    #[test]
    fn test_sync_native() {
        let program_id = crate::id();
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
            initialize_mint(&TOKEN_PROGRAM_ID, &mint_key, &owner_key, None, 2).unwrap(),
            vec![&mut mint_account, &mut rent_sysvar],
        )
        .unwrap();

        // initialize non-native account
        do_process_instruction(
            initialize_account(&TOKEN_PROGRAM_ID, &non_native_account_key, &mint_key, &owner_key)
                .unwrap(),
            vec![
                &mut non_native_account,
                &mut mint_account,
                &mut owner_account,
                &mut rent_sysvar,
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
                sync_native(&TOKEN_PROGRAM_ID, &non_native_account_key,).unwrap(),
                vec![&mut non_native_account],
            )
        );

        // fail sync uninitialized
        assert_eq!(
            Err(ProgramError::UninitializedAccount),
            do_process_instruction(
                sync_native(&TOKEN_PROGRAM_ID, &native_account_key,).unwrap(),
                vec![&mut native_account],
            )
        );

        // wrap native account
        do_process_instruction(
            initialize_account(
                &program_id,
                &native_account_key,
                &crate::native_mint::id(),
                &owner_key,
            )
            .unwrap(),
            vec![
                &mut native_account,
                &mut mint_account,
                &mut owner_account,
                &mut rent_sysvar,
            ],
        )
        .unwrap();

        // fail sync, not owned by program
        let not_program_id = Pubkey::new_unique();
        native_account.owner = not_program_id;
        assert_eq!(
            Err(ProgramError::IncorrectProgramId),
            do_process_instruction(
                sync_native(&TOKEN_PROGRAM_ID, &native_account_key,).unwrap(),
                vec![&mut native_account],
            )
        );
        native_account.owner = program_id;

        let account = Account::unpack_unchecked(&native_account.data).unwrap();
        assert!(account.is_native());
        assert_eq!(account.amount, lamports);

        // sync, no change
        do_process_instruction(
            sync_native(&TOKEN_PROGRAM_ID, &native_account_key).unwrap(),
            vec![&mut native_account],
        )
        .unwrap();
        let account = Account::unpack_unchecked(&native_account.data).unwrap();
        assert_eq!(account.amount, lamports);

        // transfer sol
        let new_lamports = lamports + 50;
        native_account.lamports = account_minimum_balance() + new_lamports;

        // success sync
        do_process_instruction(
            sync_native(&TOKEN_PROGRAM_ID, &native_account_key).unwrap(),
            vec![&mut native_account],
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
                sync_native(&TOKEN_PROGRAM_ID, &native_account_key,).unwrap(),
                vec![&mut native_account],
            )
        );
    }

    #[test]
    #[serial]
    fn test_get_account_data_size() {
        // see integration tests for return-data validity
        let program_id = crate::id();
        let owner_key = Pubkey::new_unique();
        let mut rent_sysvar = rent_sysvar();
        let mut mint_account =
            SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
        let mint_key = Pubkey::new_unique();
        // fail if an invalid mint is passed in
        assert_eq!(
            Err(TokenError::InvalidMint.into()),
            do_process_instruction(
                get_account_data_size(&TOKEN_PROGRAM_ID, &mint_key).unwrap(),
                vec![&mut mint_account],
            )
        );

        do_process_instruction(
            initialize_mint(&TOKEN_PROGRAM_ID, &mint_key, &owner_key, None, 2).unwrap(),
            vec![&mut mint_account, &mut rent_sysvar],
        )
        .unwrap();

        set_expected_data(Account::LEN.to_le_bytes().to_vec());
        do_process_instruction(
            get_account_data_size(&TOKEN_PROGRAM_ID, &mint_key).unwrap(),
            vec![&mut mint_account],
        )
        .unwrap();
    }

    #[test]
    fn test_initialize_immutable_owner() {
        let program_id = crate::id();
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
            initialize_mint(&TOKEN_PROGRAM_ID, &mint_key, &owner_key, None, 2).unwrap(),
            vec![&mut mint_account, &mut rent_sysvar],
        )
        .unwrap();

        // success initialize immutable
        do_process_instruction(
            initialize_immutable_owner(&TOKEN_PROGRAM_ID, &account_key).unwrap(),
            vec![&mut account_account],
        )
        .unwrap();

        // create account
        do_process_instruction(
            initialize_account(&TOKEN_PROGRAM_ID, &account_key, &mint_key, &owner_key).unwrap(),
            vec![
                &mut account_account,
                &mut mint_account,
                &mut owner_account,
                &mut rent_sysvar,
            ],
        )
        .unwrap();

        // fail post-init
        assert_eq!(
            Err(TokenError::AlreadyInUse.into()),
            do_process_instruction(
                initialize_immutable_owner(&TOKEN_PROGRAM_ID, &account_key).unwrap(),
                vec![&mut account_account],
            )
        );
    }

    #[test]
    #[serial]
    fn test_amount_to_ui_amount() {
        let program_id = crate::id();
        let owner_key = Pubkey::new_unique();
        let mint_key = Pubkey::new_unique();
        let mut mint_account =
            SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
        let mut rent_sysvar = rent_sysvar();

        // fail if an invalid mint is passed in
        assert_eq!(
            Err(TokenError::InvalidMint.into()),
            do_process_instruction(
                amount_to_ui_amount(&TOKEN_PROGRAM_ID, &mint_key, 110).unwrap(),
                vec![&mut mint_account],
            )
        );

        // create mint
        do_process_instruction(
            initialize_mint(&TOKEN_PROGRAM_ID, &mint_key, &owner_key, None, 2).unwrap(),
            vec![&mut mint_account, &mut rent_sysvar],
        )
        .unwrap();

        set_expected_data("0.23".as_bytes().to_vec());
        do_process_instruction(
            amount_to_ui_amount(&TOKEN_PROGRAM_ID, &mint_key, 23).unwrap(),
            vec![&mut mint_account],
        )
        .unwrap();

        set_expected_data("1.1".as_bytes().to_vec());
        do_process_instruction(
            amount_to_ui_amount(&TOKEN_PROGRAM_ID, &mint_key, 110).unwrap(),
            vec![&mut mint_account],
        )
        .unwrap();

        set_expected_data("42".as_bytes().to_vec());
        do_process_instruction(
            amount_to_ui_amount(&TOKEN_PROGRAM_ID, &mint_key, 4200).unwrap(),
            vec![&mut mint_account],
        )
        .unwrap();

        set_expected_data("0".as_bytes().to_vec());
        do_process_instruction(
            amount_to_ui_amount(&TOKEN_PROGRAM_ID, &mint_key, 0).unwrap(),
            vec![&mut mint_account],
        )
        .unwrap();
    }

    #[test]
    #[serial]
    fn test_ui_amount_to_amount() {
        let program_id = crate::id();
        let owner_key = Pubkey::new_unique();
        let mint_key = Pubkey::new_unique();
        let mut mint_account =
            SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);
        let mut rent_sysvar = rent_sysvar();

        // fail if an invalid mint is passed in
        assert_eq!(
            Err(TokenError::InvalidMint.into()),
            do_process_instruction(
                ui_amount_to_amount(&TOKEN_PROGRAM_ID, &mint_key, "1.1").unwrap(),
                vec![&mut mint_account],
            )
        );

        // create mint
        do_process_instruction(
            initialize_mint(&TOKEN_PROGRAM_ID, &mint_key, &owner_key, None, 2).unwrap(),
            vec![&mut mint_account, &mut rent_sysvar],
        )
        .unwrap();

        set_expected_data(23u64.to_le_bytes().to_vec());
        do_process_instruction(
            ui_amount_to_amount(&TOKEN_PROGRAM_ID, &mint_key, "0.23").unwrap(),
            vec![&mut mint_account],
        )
        .unwrap();

        set_expected_data(20u64.to_le_bytes().to_vec());
        do_process_instruction(
            ui_amount_to_amount(&TOKEN_PROGRAM_ID, &mint_key, "0.20").unwrap(),
            vec![&mut mint_account],
        )
        .unwrap();

        set_expected_data(20u64.to_le_bytes().to_vec());
        do_process_instruction(
            ui_amount_to_amount(&TOKEN_PROGRAM_ID, &mint_key, "0.2000").unwrap(),
            vec![&mut mint_account],
        )
        .unwrap();

        set_expected_data(20u64.to_le_bytes().to_vec());
        do_process_instruction(
            ui_amount_to_amount(&TOKEN_PROGRAM_ID, &mint_key, ".20").unwrap(),
            vec![&mut mint_account],
        )
        .unwrap();

        set_expected_data(110u64.to_le_bytes().to_vec());
        do_process_instruction(
            ui_amount_to_amount(&TOKEN_PROGRAM_ID, &mint_key, "1.1").unwrap(),
            vec![&mut mint_account],
        )
        .unwrap();

        set_expected_data(110u64.to_le_bytes().to_vec());
        do_process_instruction(
            ui_amount_to_amount(&TOKEN_PROGRAM_ID, &mint_key, "1.10").unwrap(),
            vec![&mut mint_account],
        )
        .unwrap();

        set_expected_data(4200u64.to_le_bytes().to_vec());
        do_process_instruction(
            ui_amount_to_amount(&TOKEN_PROGRAM_ID, &mint_key, "42").unwrap(),
            vec![&mut mint_account],
        )
        .unwrap();

        set_expected_data(4200u64.to_le_bytes().to_vec());
        do_process_instruction(
            ui_amount_to_amount(&TOKEN_PROGRAM_ID, &mint_key, "42.").unwrap(),
            vec![&mut mint_account],
        )
        .unwrap();

        set_expected_data(0u64.to_le_bytes().to_vec());
        do_process_instruction(
            ui_amount_to_amount(&TOKEN_PROGRAM_ID, &mint_key, "0").unwrap(),
            vec![&mut mint_account],
        )
        .unwrap();

        // fail if invalid ui_amount passed in
        assert_eq!(
            Err(ProgramError::InvalidArgument),
            do_process_instruction(
                ui_amount_to_amount(&TOKEN_PROGRAM_ID, &mint_key, "").unwrap(),
                vec![&mut mint_account],
            )
        );
        assert_eq!(
            Err(ProgramError::InvalidArgument),
            do_process_instruction(
                ui_amount_to_amount(&TOKEN_PROGRAM_ID, &mint_key, ".").unwrap(),
                vec![&mut mint_account],
            )
        );
        assert_eq!(
            Err(ProgramError::InvalidArgument),
            do_process_instruction(
                ui_amount_to_amount(&TOKEN_PROGRAM_ID, &mint_key, "0.111").unwrap(),
                vec![&mut mint_account],
            )
        );
        assert_eq!(
            Err(ProgramError::InvalidArgument),
            do_process_instruction(
                ui_amount_to_amount(&TOKEN_PROGRAM_ID, &mint_key, "0.t").unwrap(),
                vec![&mut mint_account],
            )
        );
    }
*/
