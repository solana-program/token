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
        program_pack::Pack,
        pubkey::Pubkey,
        rent::Rent,
    },
    spl_token::{
        error::TokenError,
        instruction::{initialize_account, initialize_mint, initialize_mint2},
        state::{Account, Mint},
    },
    std::collections::HashSet,
};

type InstructionPack<'a> = (Instruction, Vec<&'a SolanaAccount>);

fn do_process_instructions(
    instructions: &[InstructionPack],
    checks: &[Check],
) -> InstructionResult {
    let mut present = HashSet::new();
    let mut tx_instructions = Vec::new();
    let mut tx_accounts = Vec::new();

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
                if !present.contains(&pubkey) {
                    present.insert(pubkey);
                    tx_accounts.push((pubkey, account));
                }
            });
        tx_instructions.push(instruction.clone());
    });

    let mollusk = Mollusk::new(&spl_token::id(), "spl_token");
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

fn rent_sysvar() -> SolanaAccount {
    create_account_for_test(&Rent::default())
}

#[test]
fn test_initialize_mint() {
    let program_id = spl_token::id();
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
            initialize_mint(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
            vec![&mint_account, &rent_sysvar],
        )],
        &[Check::err(TokenError::NotRentExempt.into())],
    );

    mint_account.lamports = mint_minimum_balance();

    // create new mint
    do_process_instructions(
        &[(
            initialize_mint(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
            vec![&mint_account, &rent_sysvar],
        )],
        &[Check::success()],
    );

    // create twice
    do_process_instructions(
        &[
            (
                initialize_mint(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
                vec![&mint_account, &rent_sysvar],
            ),
            (
                initialize_mint(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
                vec![&mint_account, &rent_sysvar],
            ),
        ],
        &[Check::err(TokenError::AlreadyInUse.into())],
    );

    // create another mint that can freeze
    do_process_instructions(
        &[(
            initialize_mint(&program_id, &mint2_key, &owner_key, Some(&owner_key), 2).unwrap(),
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
    let program_id = spl_token::id();
    let owner_key = Pubkey::new_unique();
    let mint_key = Pubkey::new_unique();
    let mut mint_account = SolanaAccount::new(42, Mint::get_packed_len(), &program_id);
    let mint2_key = Pubkey::new_unique();
    let mut mint2_account =
        SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);

    // mint is not rent exempt
    do_process_instructions(
        &[(
            initialize_mint2(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
            vec![&mint_account],
        )],
        &[Check::err(TokenError::NotRentExempt.into())],
    );

    mint_account.lamports = mint_minimum_balance();

    // create new mint
    do_process_instructions(
        &[(
            initialize_mint2(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
            vec![&mint_account],
        )],
        &[Check::success()],
    );

    // create twice
    do_process_instructions(
        &[
            (
                initialize_mint2(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
                vec![&mint_account],
            ),
            (
                initialize_mint2(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
                vec![&mint_account],
            ),
        ],
        &[Check::err(TokenError::AlreadyInUse.into())],
    );

    // create another mint that can freeze
    do_process_instructions(
        &[(
            initialize_mint2(&program_id, &mint2_key, &owner_key, Some(&owner_key), 2).unwrap(),
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
    let program_id = spl_token::id();
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
            initialize_account(&program_id, &account_key, &mint_key, &owner_key).unwrap(),
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
            initialize_account(&program_id, &account_key, &mint_key, &owner_key).unwrap(),
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
            initialize_mint(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
            vec![&mint_account, &rent_sysvar],
        )],
        &[Check::success()],
    );

    // mint not owned by program
    let not_program_id = Pubkey::new_unique();
    mint_account.owner = not_program_id;

    do_process_instructions(
        &[(
            initialize_account(&program_id, &account_key, &mint_key, &owner_key).unwrap(),
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
                initialize_mint(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
                vec![&mint_account, &rent_sysvar],
            ),
            (
                initialize_account(&program_id, &account_key, &mint_key, &owner_key).unwrap(),
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
                initialize_mint(&program_id, &mint_key, &owner_key, None, 2).unwrap(),
                vec![&mint_account, &rent_sysvar],
            ),
            (
                initialize_account(&program_id, &account_key, &mint_key, &owner_key).unwrap(),
                vec![
                    &account_account,
                    &mint_account,
                    &owner_account,
                    &rent_sysvar,
                ],
            ),
            (
                initialize_account(&program_id, &account_key, &mint_key, &owner_key).unwrap(),
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
