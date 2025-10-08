mod setup;

use {
    crate::setup::TOKEN_PROGRAM_ID,
    mollusk_svm::{result::Check, Mollusk},
    pinocchio_token_interface::{
        error::TokenError,
        instruction::TokenInstruction,
        native_mint,
        state::{
            account::Account as TokenAccount, account_state::AccountState, load_mut_unchecked,
        },
    },
    solana_account::Account,
    solana_instruction::{error::InstructionError, AccountMeta, Instruction},
    solana_program_error::ProgramError,
    solana_program_pack::Pack,
    solana_pubkey::Pubkey,
    solana_rent::Rent,
    solana_sdk_ids::bpf_loader_upgradeable,
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

/// Creates a Mollusk instance with the default feature set.
fn mollusk() -> Mollusk {
    let mut mollusk = Mollusk::default();
    mollusk.add_program(
        &TOKEN_PROGRAM_ID,
        "pinocchio_token_program",
        &bpf_loader_upgradeable::id(),
    );
    mollusk
}

fn unwrap_lamports_instruction(
    source: &Pubkey,
    destination: &Pubkey,
    authority: &Pubkey,
    amount: Option<u64>,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new(*source, false),
        AccountMeta::new(*destination, false),
        AccountMeta::new_readonly(*authority, true),
    ];

    // Start with the batch discriminator
    let mut data: Vec<u8> = vec![TokenInstruction::UnwrapLamports as u8];

    if let Some(amount) = amount {
        data.push(1);
        data.extend_from_slice(&amount.to_le_bytes());
    } else {
        data.push(0);
    }

    Ok(Instruction {
        program_id: spl_token_interface::ID,
        data,
        accounts,
    })
}

#[test]
fn unwrap_lamports() {
    let native_mint = Pubkey::new_from_array(native_mint::ID);
    let authority_key = Pubkey::new_unique();
    let destination_account_key = Pubkey::new_unique();

    // native account:
    //   - amount: 2_000_000_000
    let source_account_key = Pubkey::new_unique();
    let source_account = create_token_account(
        &native_mint,
        &authority_key,
        true,
        2_000_000_000,
        &TOKEN_PROGRAM_ID,
    );

    let instruction = unwrap_lamports_instruction(
        &source_account_key,
        &destination_account_key,
        &authority_key,
        None,
    )
    .unwrap();

    // It should succeed to unwrap 2_000_000_000 lamports.

    let result = mollusk().process_and_validate_instruction(
        &instruction,
        &[
            (source_account_key, source_account),
            (destination_account_key, Account::default()),
            (authority_key, Account::default()),
        ],
        &[
            Check::success(),
            Check::account(&destination_account_key)
                .lamports(2_000_000_000)
                .build(),
            Check::account(&source_account_key)
                .lamports(Rent::default().minimum_balance(size_of::<TokenAccount>()))
                .build(),
        ],
    );

    // And the remaining amount must be 0.

    let account = result.get_account(&source_account_key);
    assert!(account.is_some());

    let account = account.unwrap();
    let token_account = spl_token_interface::state::Account::unpack(&account.data).unwrap();
    assert_eq!(token_account.amount, 0);
}

#[test]
fn unwrap_lamports_with_amount() {
    let native_mint = Pubkey::new_from_array(native_mint::ID);
    let authority_key = Pubkey::new_unique();
    let destination_account_key = Pubkey::new_unique();

    // native account:
    //   - amount: 2_000_000_000
    let source_account_key = Pubkey::new_unique();
    let source_account = create_token_account(
        &native_mint,
        &authority_key,
        true,
        2_000_000_000,
        &TOKEN_PROGRAM_ID,
    );

    let instruction = unwrap_lamports_instruction(
        &source_account_key,
        &destination_account_key,
        &authority_key,
        Some(2_000_000_000),
    )
    .unwrap();

    // It should succeed to unwrap 2_000_000_000 lamports.

    let result = mollusk().process_and_validate_instruction(
        &instruction,
        &[
            (source_account_key, source_account),
            (destination_account_key, Account::default()),
            (authority_key, Account::default()),
        ],
        &[
            Check::success(),
            Check::account(&destination_account_key)
                .lamports(2_000_000_000)
                .build(),
            Check::account(&source_account_key)
                .lamports(Rent::default().minimum_balance(size_of::<TokenAccount>()))
                .build(),
        ],
    );

    // And the remaining amount must be 0.

    let account = result.get_account(&source_account_key);
    assert!(account.is_some());

    let account = account.unwrap();
    let token_account = spl_token_interface::state::Account::unpack(&account.data).unwrap();
    assert_eq!(token_account.amount, 0);
}

#[test]
fn fail_unwrap_lamports_with_insufficient_funds() {
    let native_mint = Pubkey::new_from_array(native_mint::ID);
    let authority_key = Pubkey::new_unique();
    let destination_account_key = Pubkey::new_unique();

    // native account:
    //   - amount: 1_000_000_000
    let source_account_key = Pubkey::new_unique();
    let source_account = create_token_account(
        &native_mint,
        &authority_key,
        true,
        1_000_000_000,
        &TOKEN_PROGRAM_ID,
    );

    let instruction = unwrap_lamports_instruction(
        &source_account_key,
        &destination_account_key,
        &authority_key,
        Some(2_000_000_000),
    )
    .unwrap();

    // When we try to unwrap 2_000_000_000 lamports, we expect a
    // `TokenError::InsufficientFunds` error.

    mollusk().process_and_validate_instruction(
        &instruction,
        &[
            (source_account_key, source_account),
            (destination_account_key, Account::default()),
            (authority_key, Account::default()),
        ],
        &[Check::err(ProgramError::Custom(
            TokenError::InsufficientFunds as u32,
        ))],
    );
}

#[test]
fn unwrap_lamports_with_parial_amount() {
    let native_mint = Pubkey::new_from_array(native_mint::ID);
    let authority_key = Pubkey::new_unique();
    let destination_account_key = Pubkey::new_unique();

    // native account:
    //   - amount: 2_000_000_000
    let source_account_key = Pubkey::new_unique();
    let source_account = create_token_account(
        &native_mint,
        &authority_key,
        true,
        2_000_000_000,
        &TOKEN_PROGRAM_ID,
    );

    let instruction = unwrap_lamports_instruction(
        &source_account_key,
        &destination_account_key,
        &authority_key,
        Some(1_000_000_000),
    )
    .unwrap();

    // It should succeed to unwrap 1_000_000_000 lamports.

    let result = mollusk().process_and_validate_instruction(
        &instruction,
        &[
            (source_account_key, source_account),
            (destination_account_key, Account::default()),
            (authority_key, Account::default()),
        ],
        &[
            Check::success(),
            Check::account(&destination_account_key)
                .lamports(1_000_000_000)
                .build(),
            Check::account(&source_account_key)
                .lamports(
                    Rent::default().minimum_balance(size_of::<TokenAccount>()) + 1_000_000_000,
                )
                .build(),
        ],
    );

    // And the remaining amount must be 1_000_000_000.

    let account = result.get_account(&source_account_key);
    assert!(account.is_some());

    let account = account.unwrap();
    let token_account = spl_token_interface::state::Account::unpack(&account.data).unwrap();
    assert_eq!(token_account.amount, 1_000_000_000);
}

#[test]
fn fail_unwrap_lamports_with_invalid_authority() {
    let native_mint = Pubkey::new_from_array(native_mint::ID);
    let authority_key = Pubkey::new_unique();
    let destination_account_key = Pubkey::new_unique();
    let fake_authority_key = Pubkey::new_unique();

    // native account:
    //   - amount: 1_000_000_000
    let source_account_key = Pubkey::new_unique();
    let source_account = create_token_account(
        &native_mint,
        &authority_key,
        true,
        1_000_000_000,
        &TOKEN_PROGRAM_ID,
    );

    let instruction = unwrap_lamports_instruction(
        &source_account_key,
        &destination_account_key,
        &fake_authority_key, // <-- wrong authority
        Some(2_000_000_000),
    )
    .unwrap();

    // When we try to unwrap lamports with an invalid authority, we expect a
    // `TokenError::OwnerMismatch` error.

    mollusk().process_and_validate_instruction(
        &instruction,
        &[
            (source_account_key, source_account),
            (destination_account_key, Account::default()),
            (fake_authority_key, Account::default()),
        ],
        &[Check::err(ProgramError::Custom(
            TokenError::OwnerMismatch as u32,
        ))],
    );
}

#[test]
fn fail_unwrap_lamports_with_non_native_account() {
    let mint = Pubkey::new_unique();
    let authority_key = Pubkey::new_unique();
    let destination_account_key = Pubkey::new_unique();

    // non-native account:
    //   - amount: 2_000_000_000
    let source_account_key = Pubkey::new_unique();
    let mut source_account = create_token_account(
        &mint,
        &authority_key,
        false, // <-- non-native account
        2_000_000_000,
        &TOKEN_PROGRAM_ID,
    );
    source_account.lamports += 2_000_000_000;

    let instruction = unwrap_lamports_instruction(
        &source_account_key,
        &destination_account_key,
        &authority_key,
        Some(1_000_000_000),
    )
    .unwrap();

    // When we try to unwrap lamports from a non-native account, we expect a
    // `TokenError::NonNativeNotSupported` error.

    mollusk().process_and_validate_instruction(
        &instruction,
        &[
            (source_account_key, source_account),
            (destination_account_key, Account::default()),
            (authority_key, Account::default()),
        ],
        &[Check::err(ProgramError::Custom(
            TokenError::NonNativeNotSupported as u32,
        ))],
    );
}

#[test]
fn unwrap_lamports_with_self_transfer() {
    let native_mint = Pubkey::new_from_array(native_mint::ID);
    let authority_key = Pubkey::new_unique();

    // native account:
    //   - amount: 2_000_000_000
    let source_account_key = Pubkey::new_unique();
    let source_account = create_token_account(
        &native_mint,
        &authority_key,
        true,
        2_000_000_000,
        &TOKEN_PROGRAM_ID,
    );

    let instruction = unwrap_lamports_instruction(
        &source_account_key,
        &source_account_key, // <-- destination same as source
        &authority_key,
        Some(1_000_000_000),
    )
    .unwrap();

    // It should succeed to unwrap lamports with the same source and destination
    // accounts.

    let result = mollusk().process_and_validate_instruction(
        &instruction,
        &[
            (source_account_key, source_account),
            (authority_key, Account::default()),
        ],
        &[
            Check::success(),
            Check::account(&source_account_key)
                .lamports(
                    Rent::default().minimum_balance(size_of::<TokenAccount>()) + 2_000_000_000,
                )
                .build(),
        ],
    );

    let account = result.get_account(&source_account_key);
    assert!(account.is_some());

    let account = account.unwrap();
    let token_account = spl_token_interface::state::Account::unpack(&account.data).unwrap();
    assert_eq!(token_account.amount, 2_000_000_000);
}

#[test]
fn fail_unwrap_lamports_with_invalid_native_account() {
    let native_mint = Pubkey::new_from_array(native_mint::ID);
    let authority_key = Pubkey::new_unique();
    let destination_account_key = Pubkey::new_unique();
    let invalid_program_owner = Pubkey::new_unique();

    // native account:
    //   - amount: 2_000_000_000
    let source_account_key = Pubkey::new_unique();
    let mut source_account = create_token_account(
        &native_mint,
        &authority_key,
        true,
        2_000_000_000,
        &invalid_program_owner, // <-- invalid program owner
    );
    source_account.lamports += 2_000_000_000;

    let instruction = unwrap_lamports_instruction(
        &source_account_key,
        &destination_account_key,
        &authority_key,
        Some(1_000_000_000),
    )
    .unwrap();

    // When we try to unwrap lamports with an invalid native account, we expect
    // a `InstructionError::ExternalAccountDataModified` error.

    mollusk().process_and_validate_instruction(
        &instruction,
        &[
            (source_account_key, source_account),
            (destination_account_key, Account::default()),
            (authority_key, Account::default()),
        ],
        &[Check::instruction_err(
            InstructionError::ExternalAccountDataModified,
        )],
    );
}

#[test]
fn unwrap_lamports_to_native_account() {
    let native_mint = Pubkey::new_from_array(native_mint::ID);
    let authority_key = Pubkey::new_unique();

    // native account:
    //   - amount: 2_000_000_000
    let source_account_key = Pubkey::new_unique();
    let source_account = create_token_account(
        &native_mint,
        &authority_key,
        true,
        2_000_000_000,
        &TOKEN_PROGRAM_ID,
    );

    // destination native account:
    //   - amount: 0
    let destination_account_key = Pubkey::new_unique();
    let destination_account =
        create_token_account(&native_mint, &authority_key, true, 0, &TOKEN_PROGRAM_ID);

    let instruction = unwrap_lamports_instruction(
        &source_account_key,
        &destination_account_key,
        &authority_key,
        None,
    )
    .unwrap();

    // It should succeed to unwrap 2_000_000_000 lamports.

    let result = mollusk().process_and_validate_instruction(
        &instruction,
        &[
            (source_account_key, source_account),
            (destination_account_key, destination_account),
            (authority_key, Account::default()),
        ],
        &[
            Check::success(),
            Check::account(&destination_account_key)
                .lamports(
                    Rent::default().minimum_balance(size_of::<TokenAccount>()) + 2_000_000_000,
                )
                .build(),
            Check::account(&source_account_key)
                .lamports(Rent::default().minimum_balance(size_of::<TokenAccount>()))
                .build(),
        ],
    );

    // And the remaining amount on the source account must be 0.

    let account = result.get_account(&source_account_key);
    assert!(account.is_some());

    let account = account.unwrap();
    let token_account = spl_token_interface::state::Account::unpack(&account.data).unwrap();
    assert_eq!(token_account.amount, 0);

    // And the amount on the destination account must be 0 since we transferred
    // lamports directly to the account.

    let account = result.get_account(&destination_account_key);
    assert!(account.is_some());

    let account = account.unwrap();
    let token_account = spl_token_interface::state::Account::unpack(&account.data).unwrap();
    assert_eq!(token_account.amount, 0);
}

#[test]
fn unwrap_lamports_to_token_account() {
    let native_mint = Pubkey::new_from_array(native_mint::ID);
    let authority_key = Pubkey::new_unique();
    let non_native_mint = Pubkey::new_unique();

    // native account:
    //   - amount: 2_000_000_000
    let source_account_key = Pubkey::new_unique();
    let source_account = create_token_account(
        &native_mint,
        &authority_key,
        true,
        2_000_000_000,
        &TOKEN_PROGRAM_ID,
    );

    // destination non-native account:
    //   - amount: 0
    let destination_account_key = Pubkey::new_unique();
    let destination_account = create_token_account(
        &non_native_mint,
        &authority_key,
        false,
        0,
        &TOKEN_PROGRAM_ID,
    );

    let instruction = unwrap_lamports_instruction(
        &source_account_key,
        &destination_account_key,
        &authority_key,
        None,
    )
    .unwrap();

    // It should succeed to unwrap 2_000_000_000 lamports.

    let result = mollusk().process_and_validate_instruction(
        &instruction,
        &[
            (source_account_key, source_account),
            (destination_account_key, destination_account),
            (authority_key, Account::default()),
        ],
        &[
            Check::success(),
            Check::account(&destination_account_key)
                .lamports(
                    Rent::default().minimum_balance(size_of::<TokenAccount>()) + 2_000_000_000,
                )
                .build(),
            Check::account(&source_account_key)
                .lamports(Rent::default().minimum_balance(size_of::<TokenAccount>()))
                .build(),
        ],
    );

    // And the remaining amount on the source account must be 0.

    let account = result.get_account(&source_account_key);
    assert!(account.is_some());

    let account = account.unwrap();
    let token_account = spl_token_interface::state::Account::unpack(&account.data).unwrap();
    assert_eq!(token_account.amount, 0);

    // And the amount on the destination account must be 0 since we transferred
    // lamports directly to the account.

    let account = result.get_account(&destination_account_key);
    assert!(account.is_some());

    let account = account.unwrap();
    let token_account = spl_token_interface::state::Account::unpack(&account.data).unwrap();
    assert_eq!(token_account.amount, 0);
}
