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
            multisig::Multisig,
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
    delegate_and_amount: Option<(&Pubkey, u64)>,
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

    if let Some((delegate, delegated_amount)) = delegate_and_amount {
        token.set_delegate(delegate.as_array());
        token.set_delegated_amount(delegated_amount);
    }

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
    signers: &[&Pubkey],
) -> Result<Instruction, ProgramError> {
    let mut accounts = vec![
        AccountMeta::new(*source, false),
        AccountMeta::new(*destination, false),
        AccountMeta::new_readonly(*authority, true),
    ];

    for signer in signers {
        accounts.push(AccountMeta::new_readonly(**signer, true))
    }

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
        None,
    );

    let instruction = unwrap_lamports_instruction(
        &source_account_key,
        &destination_account_key,
        &authority_key,
        None,
        &[],
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
        None,
    );

    let instruction = unwrap_lamports_instruction(
        &source_account_key,
        &destination_account_key,
        &authority_key,
        Some(2_000_000_000),
        &[],
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
        None,
    );

    let instruction = unwrap_lamports_instruction(
        &source_account_key,
        &destination_account_key,
        &authority_key,
        Some(2_000_000_000),
        &[],
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
        None,
    );

    let instruction = unwrap_lamports_instruction(
        &source_account_key,
        &destination_account_key,
        &authority_key,
        Some(1_000_000_000),
        &[],
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
        None,
    );

    let instruction = unwrap_lamports_instruction(
        &source_account_key,
        &destination_account_key,
        &fake_authority_key, // <-- wrong authority
        Some(1_000_000_000),
        &[],
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
        None,
    );
    source_account.lamports += 2_000_000_000;

    let instruction = unwrap_lamports_instruction(
        &source_account_key,
        &destination_account_key,
        &authority_key,
        Some(1_000_000_000),
        &[],
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
        None,
    );

    let instruction = unwrap_lamports_instruction(
        &source_account_key,
        &source_account_key, // <-- destination same as source
        &authority_key,
        Some(1_000_000_000),
        &[],
    )
    .unwrap();

    // It should succeed to unwrap lamports with the same source and destination
    // accounts. The amount should be deducted from the source account but the
    // lamports should remain the same.

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
    assert_eq!(token_account.amount, 1_000_000_000);
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
        None,
    );
    source_account.lamports += 2_000_000_000;

    let instruction = unwrap_lamports_instruction(
        &source_account_key,
        &destination_account_key,
        &authority_key,
        Some(1_000_000_000),
        &[],
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
        None,
    );

    // destination native account:
    //   - amount: 0
    let destination_account_key = Pubkey::new_unique();
    let destination_account = create_token_account(
        &native_mint,
        &authority_key,
        true,
        0,
        &TOKEN_PROGRAM_ID,
        None,
    );

    let instruction = unwrap_lamports_instruction(
        &source_account_key,
        &destination_account_key,
        &authority_key,
        None,
        &[],
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
        None,
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
        None,
    );

    let instruction = unwrap_lamports_instruction(
        &source_account_key,
        &destination_account_key,
        &authority_key,
        None,
        &[],
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
fn unwrap_lamports_with_delegate_and_amount() {
    let native_mint = Pubkey::new_from_array(native_mint::ID);
    let authority_key = Pubkey::new_unique();
    let delegate_key = Pubkey::new_unique();
    let destination_account_key = Pubkey::new_unique();

    // native account:
    //   - amount: 2_000_000_000
    //   - delegated_amount: 1_000_000_000
    let source_account_key = Pubkey::new_unique();
    let source_account = create_token_account(
        &native_mint,
        &authority_key,
        true,
        2_000_000_000,
        &TOKEN_PROGRAM_ID,
        Some((&delegate_key, 1_000_000_000)),
    );

    let instruction = unwrap_lamports_instruction(
        &source_account_key,
        &destination_account_key,
        &delegate_key, // <-- delegate authority
        Some(500_000_000),
        &[],
    )
    .unwrap();

    // It should succeed to unwrap 500_000_000 lamports.
    let result = mollusk().process_and_validate_instruction(
        &instruction,
        &[
            (source_account_key, source_account),
            (destination_account_key, Account::default()),
            (delegate_key, Account::default()),
        ],
        &[
            Check::success(),
            Check::account(&destination_account_key)
                .lamports(500_000_000)
                .build(),
            Check::account(&source_account_key)
                .lamports(
                    Rent::default().minimum_balance(size_of::<TokenAccount>()) + 1_500_000_000,
                )
                .build(),
        ],
    );

    // And the remaining amount must be 1_500_000_000 and the delegated amount reduced
    // to 500_000_000.

    let account = result.get_account(&source_account_key);
    assert!(account.is_some());

    let account = account.unwrap();
    let token_account = spl_token_interface::state::Account::unpack(&account.data).unwrap();
    assert_eq!(token_account.amount, 1_500_000_000);
    assert_eq!(token_account.delegate.unwrap(), delegate_key);
    assert_eq!(token_account.delegated_amount, 500_000_000);
}

#[test]
fn unwrap_lamports_with_delegate() {
    let native_mint = Pubkey::new_from_array(native_mint::ID);
    let authority_key = Pubkey::new_unique();
    let delegate_key = Pubkey::new_unique();
    let destination_account_key = Pubkey::new_unique();

    // native account:
    //   - amount: 2_000_000_000
    //   - delegated_amount: 2_000_000_000
    let source_account_key = Pubkey::new_unique();
    let source_account = create_token_account(
        &native_mint,
        &authority_key,
        true,
        2_000_000_000,
        &TOKEN_PROGRAM_ID,
        Some((&delegate_key, 2_000_000_000)),
    );

    let instruction = unwrap_lamports_instruction(
        &source_account_key,
        &destination_account_key,
        &delegate_key, // <-- delegate authority
        None,          // <-- unwrap full amount
        &[],
    )
    .unwrap();

    // It should succeed to unwrap 2_000_000_000 lamports.

    let result = mollusk().process_and_validate_instruction(
        &instruction,
        &[
            (source_account_key, source_account),
            (destination_account_key, Account::default()),
            (delegate_key, Account::default()),
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

    // And the remaining amount must be 0 and the delegate cleared.

    let account = result.get_account(&source_account_key);
    assert!(account.is_some());

    let account = account.unwrap();
    let token_account = spl_token_interface::state::Account::unpack(&account.data).unwrap();
    assert_eq!(token_account.amount, 0);
    assert!(token_account.delegate.is_none());
    assert_eq!(token_account.delegated_amount, 0);
}

#[test]
fn fail_unwrap_lamports_with_delegate_and_insufficient_amount() {
    let native_mint = Pubkey::new_from_array(native_mint::ID);
    let authority_key = Pubkey::new_unique();
    let delegate_key = Pubkey::new_unique();
    let destination_account_key = Pubkey::new_unique();

    // native account:
    //   - amount: 2_000_000_000
    //   - delegated_amount: 1_000_000_000
    let source_account_key = Pubkey::new_unique();
    let source_account = create_token_account(
        &native_mint,
        &authority_key,
        true,
        2_000_000_000,
        &TOKEN_PROGRAM_ID,
        Some((&delegate_key, 1_000_000_000)),
    );

    let instruction = unwrap_lamports_instruction(
        &source_account_key,
        &destination_account_key,
        &delegate_key, // <-- delegate authority
        None,
        &[],
    )
    .unwrap();

    // When we try to unwrap 2_000_000_000 lamports, we expect a
    // `TokenError::InsufficientFunds` error since only 1_000_000_000
    // lamports are delegated.

    mollusk().process_and_validate_instruction(
        &instruction,
        &[
            (source_account_key, source_account),
            (destination_account_key, Account::default()),
            (delegate_key, Account::default()),
        ],
        &[Check::err(ProgramError::Custom(
            TokenError::InsufficientFunds as u32,
        ))],
    );
}

#[test]
fn fail_unwrap_lamports_with_wrong_delegate() {
    let native_mint = Pubkey::new_from_array(native_mint::ID);
    let authority_key = Pubkey::new_unique();
    let delegate_key = Pubkey::new_unique();
    let destination_account_key = Pubkey::new_unique();

    // native account:
    //   - amount: 2_000_000_000
    //   - delegated_amount: 1_000_000_000
    let source_account_key = Pubkey::new_unique();
    let source_account = create_token_account(
        &native_mint,
        &authority_key,
        true,
        2_000_000_000,
        &TOKEN_PROGRAM_ID,
        Some((&delegate_key, 1_000_000_000)),
    );

    let fake_delegate_key = Pubkey::new_unique();

    let instruction = unwrap_lamports_instruction(
        &source_account_key,
        &destination_account_key,
        &fake_delegate_key, // <-- fake delegate authority
        None,
        &[],
    )
    .unwrap();

    // When we try to unwrap lamports with an invalid delegate, we expect
    // a `TokenError::OwnerMismatch` error.

    mollusk().process_and_validate_instruction(
        &instruction,
        &[
            (source_account_key, source_account),
            (destination_account_key, Account::default()),
            (fake_delegate_key, Account::default()),
        ],
        &[Check::err(ProgramError::Custom(
            TokenError::OwnerMismatch as u32,
        ))],
    );
}

#[test]
fn unwrap_lamports_from_multisig() {
    let native_mint = Pubkey::new_from_array(native_mint::ID);
    let destination_account_key = Pubkey::new_unique();
    let molusk_svm = mollusk();

    // Given a multisig account.

    let multisig_key = Pubkey::new_unique();
    let signer1_key = Pubkey::new_unique();
    let signer2_key = Pubkey::new_unique();
    let signer3_key = Pubkey::new_unique();

    let initialize_multisig_ix = spl_token_interface::instruction::initialize_multisig(
        &spl_token_interface::ID,
        &multisig_key,
        &[&signer1_key, &signer2_key, &signer3_key],
        3,
    )
    .unwrap();

    let result = molusk_svm.process_and_validate_instruction(
        &initialize_multisig_ix,
        &[
            (
                multisig_key,
                Account {
                    lamports: Rent::default().minimum_balance(size_of::<Multisig>()),
                    data: vec![0u8; size_of::<Multisig>()],
                    owner: TOKEN_PROGRAM_ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            molusk_svm.sysvars.keyed_account_for_rent_sysvar(),
            (signer1_key, Account::default()),
            (signer2_key, Account::default()),
            (signer3_key, Account::default()),
        ],
        &[Check::success()],
    );

    let multisig = result.get_account(&multisig_key).unwrap();

    // native account:
    //   - amount: 2_000_000_000
    //   - owner: multisig
    let source_account_key = Pubkey::new_unique();
    let source_account = create_token_account(
        &native_mint,
        &multisig_key,
        true,
        2_000_000_000,
        &TOKEN_PROGRAM_ID,
        None,
    );

    let instruction = unwrap_lamports_instruction(
        &source_account_key,
        &destination_account_key,
        &multisig_key,
        None,
        &[&signer1_key, &signer2_key, &signer3_key],
    )
    .unwrap();

    // It should succeed to unwrap 2_000_000_000 lamports.

    let result = molusk_svm.process_and_validate_instruction(
        &instruction,
        &[
            (source_account_key, source_account),
            (destination_account_key, Account::default()),
            (multisig_key, multisig.clone()),
            (signer1_key, Account::default()),
            (signer2_key, Account::default()),
            (signer3_key, Account::default()),
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
fn fail_unwrap_lamports_from_multisig_with_missing_signer() {
    let native_mint = Pubkey::new_from_array(native_mint::ID);
    let destination_account_key = Pubkey::new_unique();
    let molusk_svm = mollusk();

    // Given a multisig account.

    let multisig_key = Pubkey::new_unique();
    let signer1_key = Pubkey::new_unique();
    let signer2_key = Pubkey::new_unique();
    let signer3_key = Pubkey::new_unique();

    let initialize_multisig_ix = spl_token_interface::instruction::initialize_multisig(
        &spl_token_interface::ID,
        &multisig_key,
        &[&signer1_key, &signer2_key, &signer3_key],
        3,
    )
    .unwrap();

    let result = molusk_svm.process_and_validate_instruction(
        &initialize_multisig_ix,
        &[
            (
                multisig_key,
                Account {
                    lamports: Rent::default().minimum_balance(size_of::<Multisig>()),
                    data: vec![0u8; size_of::<Multisig>()],
                    owner: TOKEN_PROGRAM_ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            molusk_svm.sysvars.keyed_account_for_rent_sysvar(),
            (signer1_key, Account::default()),
            (signer2_key, Account::default()),
            (signer3_key, Account::default()),
        ],
        &[Check::success()],
    );

    let multisig = result.get_account(&multisig_key).unwrap();

    // native account:
    //   - amount: 2_000_000_000
    //   - owner: multisig
    let source_account_key = Pubkey::new_unique();
    let source_account = create_token_account(
        &native_mint,
        &multisig_key,
        true,
        2_000_000_000,
        &TOKEN_PROGRAM_ID,
        None,
    );

    let instruction = unwrap_lamports_instruction(
        &source_account_key,
        &destination_account_key,
        &multisig_key,
        None,
        &[&signer1_key, &signer2_key], // <-- missing signer3
    )
    .unwrap();

    // When we try to unwrap lamports with a missing signer, we expect a
    // `InstructionError::MissingRequiredSignature` error.

    molusk_svm.process_and_validate_instruction(
        &instruction,
        &[
            (source_account_key, source_account),
            (destination_account_key, Account::default()),
            (multisig_key, multisig.clone()),
            (signer1_key, Account::default()),
            (signer2_key, Account::default()),
        ],
        &[Check::err(ProgramError::MissingRequiredSignature)],
    );
}
