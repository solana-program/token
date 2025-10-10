mod setup;

use {
    crate::setup::TOKEN_PROGRAM_ID,
    agave_feature_set::FeatureSet,
    mollusk_svm::{result::Check, Mollusk},
    pinocchio_token_interface::{
        native_mint,
        state::{
            account::Account as TokenAccount, account_state::AccountState, load_mut_unchecked,
            mint::Mint,
        },
    },
    solana_account::Account,
    solana_instruction::{AccountMeta, Instruction},
    solana_keypair::Keypair,
    solana_program_error::ProgramError,
    solana_program_pack::Pack,
    solana_program_test::{tokio, ProgramTest},
    solana_pubkey::Pubkey,
    solana_rent::Rent,
    solana_sdk_ids::bpf_loader_upgradeable,
    solana_signer::Signer,
    solana_system_interface::instruction::create_account,
    solana_transaction::Transaction,
};

fn batch_instruction(instructions: Vec<Instruction>) -> Result<Instruction, ProgramError> {
    // Create a `Vec` of ordered `AccountMeta`s
    let mut accounts: Vec<AccountMeta> = vec![];
    // Start with the batch discriminator
    let mut data: Vec<u8> = vec![0xff];

    for instruction in instructions {
        // Error out on non-token IX.
        if instruction.program_id.ne(&spl_token_interface::ID) {
            return Err(ProgramError::IncorrectProgramId);
        }

        data.push(instruction.accounts.len() as u8);
        data.push(instruction.data.len() as u8);

        data.extend_from_slice(&instruction.data);
        accounts.extend_from_slice(&instruction.accounts);
    }

    Ok(Instruction {
        program_id: spl_token_interface::ID,
        data,
        accounts,
    })
}

#[tokio::test]
async fn batch_initialize_mint_transfer_close() {
    let context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    let rent = context.banks_client.get_rent().await.unwrap();

    let mint_len = spl_token_interface::state::Mint::LEN;
    let mint_rent = rent.minimum_balance(mint_len);

    let account_len = spl_token_interface::state::Account::LEN;
    let account_rent = rent.minimum_balance(account_len);

    // Create a mint
    let mint_a = Keypair::new();
    let mint_authority = Keypair::new();
    let create_mint_a = create_account(
        &context.payer.pubkey(),
        &mint_a.pubkey(),
        mint_rent,
        mint_len as u64,
        &TOKEN_PROGRAM_ID,
    );
    let initialize_mint_ix = spl_token_interface::instruction::initialize_mint(
        &TOKEN_PROGRAM_ID,
        &mint_a.pubkey(),
        &mint_authority.pubkey(),
        None,
        6,
    )
    .unwrap();

    // Create a mint 2 with a freeze authority
    let mint_b = Keypair::new();
    let freeze_authority = Pubkey::new_unique();
    let create_mint_b = create_account(
        &context.payer.pubkey(),
        &mint_b.pubkey(),
        mint_rent,
        mint_len as u64,
        &TOKEN_PROGRAM_ID,
    );
    let initialize_mint_with_freeze_authority_ix =
        spl_token_interface::instruction::initialize_mint2(
            &TOKEN_PROGRAM_ID,
            &mint_b.pubkey(),
            &mint_authority.pubkey(),
            Some(&freeze_authority),
            6,
        )
        .unwrap();

    // Create 2 token accounts for mint A and 1 for mint B
    let owner_a = Keypair::new();
    let owner_b = Keypair::new();
    let owner_a_ta_a = Keypair::new();
    let owner_b_ta_a = Keypair::new();

    let create_owner_a_ta_a = create_account(
        &context.payer.pubkey(),
        &owner_a_ta_a.pubkey(),
        account_rent,
        account_len as u64,
        &TOKEN_PROGRAM_ID,
    );
    let create_owner_b_ta_a = create_account(
        &context.payer.pubkey(),
        &owner_b_ta_a.pubkey(),
        account_rent,
        account_len as u64,
        &TOKEN_PROGRAM_ID,
    );
    let intialize_owner_a_ta_a = spl_token_interface::instruction::initialize_account3(
        &TOKEN_PROGRAM_ID,
        &owner_a_ta_a.pubkey(),
        &mint_a.pubkey(),
        &owner_a.pubkey(),
    )
    .unwrap();
    let intialize_owner_b_ta_a = spl_token_interface::instruction::initialize_account3(
        &TOKEN_PROGRAM_ID,
        &owner_b_ta_a.pubkey(),
        &mint_a.pubkey(),
        &owner_b.pubkey(),
    )
    .unwrap();

    // Mint Token A to Owner A
    let mint_token_a_to_owner_a = spl_token_interface::instruction::mint_to(
        &TOKEN_PROGRAM_ID,
        &mint_a.pubkey(),
        &owner_a_ta_a.pubkey(),
        &mint_authority.pubkey(),
        &[],
        1_000_000,
    )
    .unwrap();

    // Transfer Token A from Owner A to Owner B
    let transfer_token_a_to_owner_b = spl_token_interface::instruction::transfer(
        &TOKEN_PROGRAM_ID,
        &owner_a_ta_a.pubkey(),
        &owner_b_ta_a.pubkey(),
        &owner_a.pubkey(),
        &[],
        1_000_000,
    )
    .unwrap();

    // Close Token A
    let close_owner_a_ta_a = spl_token_interface::instruction::close_account(
        &TOKEN_PROGRAM_ID,
        &owner_a_ta_a.pubkey(),
        &owner_a.pubkey(),
        &owner_a.pubkey(),
        &[],
    )
    .unwrap();

    let batch_ix = batch_instruction(vec![
        initialize_mint_ix,
        initialize_mint_with_freeze_authority_ix,
        intialize_owner_a_ta_a,
        intialize_owner_b_ta_a,
        mint_token_a_to_owner_a,
        transfer_token_a_to_owner_b,
        close_owner_a_ta_a,
    ])
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[
            create_mint_a,
            create_mint_b,
            create_owner_a_ta_a,
            create_owner_b_ta_a,
            batch_ix,
        ],
        Some(&context.payer.pubkey()),
        &vec![
            &context.payer,
            &mint_a,
            &mint_b,
            &owner_a_ta_a,
            &owner_b_ta_a,
            &mint_authority,
            &owner_a,
        ],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let mint_a_account = context
        .banks_client
        .get_account(mint_a.pubkey())
        .await
        .unwrap();
    assert!(mint_a_account.is_some());
    let mint_a_account =
        spl_token_interface::state::Mint::unpack(&mint_a_account.unwrap().data).unwrap();
    assert_eq!(mint_a_account.supply, 1000000);

    let mint_b_account = context
        .banks_client
        .get_account(mint_b.pubkey())
        .await
        .unwrap();
    assert!(mint_b_account.is_some());
    let mint_b_account =
        spl_token_interface::state::Mint::unpack(&mint_b_account.unwrap().data).unwrap();
    assert_eq!(mint_b_account.supply, 0);

    let owner_b_ta_a_account = context
        .banks_client
        .get_account(owner_b_ta_a.pubkey())
        .await
        .unwrap();
    assert!(owner_b_ta_a_account.is_some());
    let owner_b_ta_a_account =
        spl_token_interface::state::Account::unpack(&owner_b_ta_a_account.unwrap().data).unwrap();
    assert_eq!(owner_b_ta_a_account.amount, 1000000);

    let closed_owner_a_ta_a = context
        .banks_client
        .get_account(owner_a_ta_a.pubkey())
        .await
        .unwrap();
    assert!(closed_owner_a_ta_a.is_none());
}

fn create_mint(
    mint_authority: &Pubkey,
    supply: u64,
    decimals: u8,
    program_owner: &Pubkey,
) -> Account {
    let space = size_of::<Mint>();
    let lamports = Rent::default().minimum_balance(space);

    let mut data: Vec<u8> = vec![0u8; space];
    let mint = unsafe { load_mut_unchecked::<Mint>(data.as_mut_slice()).unwrap() };
    mint.set_initialized();
    mint.set_supply(supply);
    mint.set_mint_authority(mint_authority.as_array());
    mint.decimals = decimals;

    Account {
        lamports,
        data,
        owner: *program_owner,
        executable: false,
        ..Default::default()
    }
}

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
    token.set_native_amount(amount);

    if is_native {
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
/// `account_data_direct_mapping` feature.
fn mollusk() -> Mollusk {
    let feature_set = {
        // When upgrading to v3.1, add this back in
        //let fs = FeatureSet::all_enabled();
        //fs.active_mut()
        //    .remove(&agave_feature_set::account_data_direct_mapping::id());
        //fs
        FeatureSet::all_enabled()
    };
    let mut mollusk = Mollusk {
        feature_set,
        ..Default::default()
    };
    mollusk.add_program(
        &TOKEN_PROGRAM_ID,
        "pinocchio_token_program",
        &bpf_loader_upgradeable::id(),
    );
    mollusk
}

#[tokio::test]
async fn batch_transfer() {
    let authority_key = Pubkey::new_unique();
    let mint_key = Pubkey::new_unique();

    // source account
    //   - amount: 1_000_000_000
    //   - mint: mint_key
    //   - is_native: false
    //   - program_id: TOKEN_PROGRAM_ID
    let source_account_key = Pubkey::new_unique();
    let source_account = create_token_account(
        &mint_key,
        &authority_key,
        false,
        1_000_000_000,
        &TOKEN_PROGRAM_ID,
    );

    // destination account
    //   - amount: 0
    //   - mint: mint_key
    //   - is_native: false
    //   - program_id: TOKEN_PROGRAM_ID
    let destination_account_key = Pubkey::new_unique();
    let destination_account =
        create_token_account(&mint_key, &authority_key, false, 0, &TOKEN_PROGRAM_ID);

    let instruction = batch_instruction(vec![spl_token_interface::instruction::transfer(
        &TOKEN_PROGRAM_ID,
        &source_account_key,
        &destination_account_key,
        &authority_key,
        &[],
        500_000_000,
    )
    .unwrap()])
    .unwrap();

    // Expected to succeed.

    mollusk().process_and_validate_instruction_chain(
        &[(&instruction, &[Check::success(), Check::all_rent_exempt()])],
        &[
            (source_account_key, source_account),
            (destination_account_key, destination_account),
            (
                authority_key,
                Account {
                    lamports: Rent::default().minimum_balance(0),
                    ..Default::default()
                },
            ),
        ],
    );
}

#[tokio::test]
async fn batch_fail_transfer_with_invalid_program_owner() {
    let invalid_program_id = Pubkey::new_from_array([2; 32]);
    let native_mint = Pubkey::new_from_array(native_mint::ID);
    let authority_key = Pubkey::new_unique();

    // source account
    //   - amount: 1_000_000_000
    //   - mint: native_mint
    //   - is_native: true
    //   - program_id: invalid_program_id
    let source_account_key = Pubkey::new_unique();
    let source_account = create_token_account(
        &native_mint,
        &authority_key,
        true,
        1_000_000_000,
        &invalid_program_id,
    );

    // destination account
    //   - amount: 0
    //   - mint: native_mint
    //   - is_native: true
    //   - program_id: TOKEN_PROGRAM_ID
    let destination_account_key = Pubkey::new_unique();
    let destination_account =
        create_token_account(&native_mint, &authority_key, true, 0, &TOKEN_PROGRAM_ID);

    let instruction = batch_instruction(vec![spl_token_interface::instruction::transfer(
        &TOKEN_PROGRAM_ID,
        &source_account_key,
        &destination_account_key,
        &authority_key,
        &[],
        500_000_000,
    )
    .unwrap()])
    .unwrap();

    // Expected to fail since source account has an invalid program owner.

    mollusk().process_and_validate_instruction_chain(
        &[(
            &instruction,
            &[
                Check::err(ProgramError::IncorrectProgramId),
                Check::all_rent_exempt(),
            ],
        )],
        &[
            (source_account_key, source_account),
            (destination_account_key, destination_account),
            (
                authority_key,
                Account {
                    lamports: Rent::default().minimum_balance(0),
                    ..Default::default()
                },
            ),
        ],
    );
}

#[tokio::test]
async fn batch_fail_transfer_checked_with_invalid_program_owner() {
    let invalid_program_id = Pubkey::new_from_array([2; 32]);
    let authority_key = Pubkey::new_unique();

    let native_mint_key = Pubkey::new_from_array(native_mint::ID);
    let native_mint = create_mint(&authority_key, 5_000_000_000, 9, &TOKEN_PROGRAM_ID);

    // source account
    //   - amount: 1_000_000_000
    //   - mint: native_mint
    //   - is_native: true
    //   - program_id: invalid_program_id
    let source_account_key = Pubkey::new_unique();
    let source_account = create_token_account(
        &native_mint_key,
        &authority_key,
        true,
        1_000_000_000,
        &invalid_program_id,
    );

    // destination account
    //   - amount: 0
    //   - mint: native_mint
    //   - is_native: true
    //   - program_id: TOKEN_PROGRAM_ID
    let destination_account_key = Pubkey::new_unique();
    let destination_account =
        create_token_account(&native_mint_key, &authority_key, true, 0, &TOKEN_PROGRAM_ID);

    let instruction = batch_instruction(vec![spl_token_interface::instruction::transfer_checked(
        &TOKEN_PROGRAM_ID,
        &source_account_key,
        &native_mint_key,
        &destination_account_key,
        &authority_key,
        &[],
        500_000_000,
        9,
    )
    .unwrap()])
    .unwrap();

    // Expected to fail since source account has an invalid program owner.

    mollusk().process_and_validate_instruction_chain(
        &[(
            &instruction,
            &[
                Check::err(ProgramError::IncorrectProgramId),
                Check::all_rent_exempt(),
            ],
        )],
        &[
            (source_account_key, source_account),
            (destination_account_key, destination_account),
            (native_mint_key, native_mint),
            (
                authority_key,
                Account {
                    lamports: Rent::default().minimum_balance(0),
                    ..Default::default()
                },
            ),
        ],
    );
}

#[tokio::test]
async fn batch_fail_swap_tokens_with_invalid_program_owner() {
    let native_mint = Pubkey::new_from_array(native_mint::ID);
    let invalid_program_id = Pubkey::new_from_array([2; 32]);
    let authority_key = Pubkey::new_unique();

    // Account A
    //   - amount: 1_000
    //   - mint: native_mint
    //   - is_native: false
    //   - program_id: invalid_program_id
    let account_a_key = Pubkey::new_unique();
    let account_a = create_token_account(
        &native_mint,
        &authority_key,
        false,
        1_000,
        &invalid_program_id,
    );

    // Account B
    //   - amount: 0
    //   - mint: native_mint
    //   - is_native: true
    //   - program_id: TOKEN_PROGRAM_ID
    let account_b_key = Pubkey::new_unique();
    let account_b = create_token_account(&native_mint, &authority_key, true, 0, &TOKEN_PROGRAM_ID);

    // Account C
    //   - amount: 0
    //   - mint: native_mint
    //   - is_native: true
    //   - program_id: TOKEN_PROGRAM_ID
    let account_c_key = Pubkey::new_unique();
    let account_c =
        create_token_account(&native_mint, &authority_key, true, 1_000, &TOKEN_PROGRAM_ID);

    // Batch instruction to swap tokens
    //   - transfer 300 from account A to account B
    //   - transfer 300 from account C to account A
    let instruction = batch_instruction(vec![
        spl_token_interface::instruction::sync_native(&TOKEN_PROGRAM_ID, &account_b_key).unwrap(),
        spl_token_interface::instruction::sync_native(&TOKEN_PROGRAM_ID, &account_c_key).unwrap(),
        spl_token_interface::instruction::transfer(
            &TOKEN_PROGRAM_ID,
            &account_a_key,
            &account_b_key,
            &authority_key,
            &[],
            300,
        )
        .unwrap(),
        spl_token_interface::instruction::transfer(
            &TOKEN_PROGRAM_ID,
            &account_c_key,
            &account_a_key,
            &authority_key,
            &[],
            300,
        )
        .unwrap(),
    ])
    .unwrap();

    // Expected to fail since account A has an invalid program owner.

    mollusk().process_and_validate_instruction_chain(
        &[(
            &instruction,
            &[
                Check::err(ProgramError::IncorrectProgramId),
                Check::all_rent_exempt(),
            ],
        )],
        &[
            (account_a_key, account_a),
            (account_b_key, account_b),
            (account_c_key, account_c),
            (
                authority_key,
                Account {
                    lamports: Rent::default().minimum_balance(0),
                    ..Default::default()
                },
            ),
        ],
    );
}

#[tokio::test]
async fn batch_fail_mint_to_with_invalid_program_owner() {
    let invalid_program_id = Pubkey::new_from_array([2; 32]);
    let authority_key = Pubkey::new_unique();

    let mint_key = Pubkey::new_unique();
    let mint = create_mint(&authority_key, 0, 0, &TOKEN_PROGRAM_ID);

    // account A (invalid)
    //   - amount: 1_000_000_000
    //   - mint: native_mint
    //   - is_native: false
    //   - program_id: invalid_program_id
    let account_a_key = Pubkey::new_unique();
    let account_a = create_token_account(&mint_key, &authority_key, false, 0, &invalid_program_id);

    // account B
    //   - amount: 0
    //   - mint: native_mint
    //   - is_native: false
    //   - program_id: TOKEN_PROGRAM_ID
    let account_b_key = Pubkey::new_unique();
    let account_b = create_token_account(&mint_key, &authority_key, false, 0, &TOKEN_PROGRAM_ID);

    let instruction = batch_instruction(vec![
        spl_token_interface::instruction::mint_to(
            &TOKEN_PROGRAM_ID,
            &mint_key,
            &account_a_key,
            &authority_key,
            &[],
            1_000_000_000,
        )
        .unwrap(),
        spl_token_interface::instruction::mint_to(
            &TOKEN_PROGRAM_ID,
            &mint_key,
            &account_b_key,
            &authority_key,
            &[],
            1_000_000_000,
        )
        .unwrap(),
    ])
    .unwrap();

    // Expected to fail since source account has an invalid program owner.

    mollusk().process_and_validate_instruction_chain(
        &[(
            &instruction,
            &[
                Check::err(ProgramError::IncorrectProgramId),
                Check::all_rent_exempt(),
            ],
        )],
        &[
            (mint_key, mint),
            (account_a_key, account_a),
            (account_b_key, account_b),
            (
                authority_key,
                Account {
                    lamports: Rent::default().minimum_balance(0),
                    ..Default::default()
                },
            ),
        ],
    );
}

#[tokio::test]
async fn batch_fail_burn_with_invalid_program_owner() {
    let invalid_program_id = Pubkey::new_from_array([2; 32]);
    let authority_key = Pubkey::new_unique();

    let mint_key = Pubkey::new_unique();
    let mint = create_mint(&authority_key, 2_000_000_000, 0, &TOKEN_PROGRAM_ID);

    // account A
    //   - amount: 0
    //   - mint: native_mint
    //   - is_native: false
    //   - program_id: TOKEN_PROGRAM_ID
    let account_a_key = Pubkey::new_unique();
    let account_a = create_token_account(&mint_key, &authority_key, false, 0, &TOKEN_PROGRAM_ID);

    // account B (invalid)
    //   - amount: 1_000_000_000
    //   - mint: native_mint
    //   - is_native: false
    //   - program_id: invalid_program_id
    let account_b_key = Pubkey::new_unique();
    let account_b = create_token_account(
        &mint_key,
        &authority_key,
        false,
        1_000_000_000,
        &invalid_program_id,
    );

    let instruction = batch_instruction(vec![
        spl_token_interface::instruction::mint_to(
            &TOKEN_PROGRAM_ID,
            &mint_key,
            &account_a_key,
            &authority_key,
            &[],
            1_000_000_000,
        )
        .unwrap(),
        spl_token_interface::instruction::mint_to(
            &TOKEN_PROGRAM_ID,
            &mint_key,
            &account_b_key,
            &authority_key,
            &[],
            1_000_000_000,
        )
        .unwrap(),
        spl_token_interface::instruction::burn(
            &TOKEN_PROGRAM_ID,
            &account_a_key,
            &mint_key,
            &authority_key,
            &[],
            1_000_000_000,
        )
        .unwrap(),
        spl_token_interface::instruction::burn(
            &TOKEN_PROGRAM_ID,
            &account_b_key,
            &mint_key,
            &authority_key,
            &[],
            1_000_000_000,
        )
        .unwrap(),
    ])
    .unwrap();

    // Expected to fail since source account has an invalid program owner.

    mollusk().process_and_validate_instruction_chain(
        &[(
            &instruction,
            &[
                Check::err(ProgramError::IncorrectProgramId),
                Check::all_rent_exempt(),
            ],
        )],
        &[
            (mint_key, mint),
            (account_a_key, account_a),
            (account_b_key, account_b),
            (
                authority_key,
                Account {
                    lamports: Rent::default().minimum_balance(0),
                    ..Default::default()
                },
            ),
        ],
    );
}
