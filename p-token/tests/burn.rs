mod setup;

use {
    setup::{account, mint, TOKEN_PROGRAM_ID},
    solana_keypair::Keypair,
    solana_program_pack::Pack,
    solana_program_test::{tokio, ProgramTest},
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::Transaction,
};

#[tokio::test]
async fn burn() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Keypair::new();
    let freeze_authority = Pubkey::new_unique();

    let mint = mint::initialize(
        &mut context,
        mint_authority.pubkey(),
        Some(freeze_authority),
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    // And a token account with 100 tokens.

    let owner = Keypair::new();

    let account =
        account::initialize(&mut context, &mint, &owner.pubkey(), &TOKEN_PROGRAM_ID).await;

    mint::mint(
        &mut context,
        &mint,
        &account,
        &mint_authority,
        100,
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    // When we burn 50 tokens.

    let burn_ix =
        spl_token::instruction::burn(&spl_token::ID, &account, &mint, &owner.pubkey(), &[], 50)
            .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[burn_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &owner],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the account should have 50 tokens remaining.

    let account = context.banks_client.get_account(account).await.unwrap();

    assert!(account.is_some());

    let account = account.unwrap();
    let account = spl_token::state::Account::unpack(&account.data).unwrap();

    assert!(account.amount == 50);
}

#[tokio::test]
async fn burn_invalid_source() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Keypair::new();
    let freeze_authority = Pubkey::new_unique();

    let mint = mint::initialize(
        &mut context,
        mint_authority.pubkey(),
        Some(freeze_authority),
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    // And a token account with 100 tokens.

    let owner = Keypair::new();

    let account =
        account::initialize(&mut context, &mint, &owner.pubkey(), &TOKEN_PROGRAM_ID).await;

    mint::mint(
        &mut context,
        &mint,
        &account,
        &mint_authority,
        100,
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    // When we burn 50 tokens.

    let burn_ix =
        spl_token::instruction::burn(&spl_token::ID, &Pubkey::new_unique(), &mint, &owner.pubkey(), &[], 50)
            .unwrap(); // Invalid sender

    let tx = Transaction::new_signed_with_payer(
        &[burn_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &owner],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::InvalidAccountData));
}

#[tokio::test]
async fn burn_frozen_source() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Keypair::new();
    let freeze_authority = Keypair::new();

    let mint = mint::initialize(
        &mut context,
        mint_authority.pubkey(),
        Some(freeze_authority.pubkey()),
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    // And a token account with 100 tokens.

    let owner = Keypair::new();

    let account =
        account::initialize(&mut context, &mint, &owner.pubkey(), &TOKEN_PROGRAM_ID).await;

    mint::mint(
        &mut context,
        &mint,
        &account,
        &mint_authority,
        100,
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    let freeze_ix = spl_token::instruction::freeze_account(
        &spl_token::ID,
        &account,
        &mint,
        &freeze_authority.pubkey(),
        &[],
    )
    .unwrap();

    let freeze_tx = Transaction::new_signed_with_payer(
        &[freeze_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &freeze_authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(freeze_tx).await.unwrap();

    // When we burn 50 tokens.

    let burn_ix =
        spl_token::instruction::burn(&spl_token::ID, &account, &mint, &owner.pubkey(), &[], 50)
            .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[burn_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &owner],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::Custom(17)));
}

#[tokio::test]
async fn burn_native() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    let mint = Pubkey::from(pinocchio_token_interface::native_mint::ID);

    // And a token account with 100 tokens.

    let owner = Keypair::new();

    let account =
        account::initialize(&mut context, &mint, &owner.pubkey(), &TOKEN_PROGRAM_ID).await;

    // When we burn 50 tokens.

    let burn_ix =
        spl_token::instruction::burn(&spl_token::ID, &account, &mint, &owner.pubkey(), &[], 50)
            .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[burn_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &owner],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::Custom(10)));
}

#[tokio::test]
async fn burn_excessive_amount() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Keypair::new();
    let freeze_authority = Pubkey::new_unique();

    let mint = mint::initialize(
        &mut context,
        mint_authority.pubkey(),
        Some(freeze_authority),
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    // And a token account with 100 tokens.

    let owner = Keypair::new();

    let account =
        account::initialize(&mut context, &mint, &owner.pubkey(), &TOKEN_PROGRAM_ID).await;

    mint::mint(
        &mut context,
        &mint,
        &account,
        &mint_authority,
        100,
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    // When we burn 50 tokens.

    let burn_ix =
        spl_token::instruction::burn(&spl_token::ID, &account, &mint, &owner.pubkey(), &[], 101)
            .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[burn_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &owner],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::Custom(1)));
}

#[tokio::test]
async fn burn_different_mint() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Keypair::new();
    let freeze_authority = Pubkey::new_unique();

    let mint = mint::initialize(
        &mut context,
        mint_authority.pubkey(),
        Some(freeze_authority),
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    let native_mint = Pubkey::from(pinocchio_token_interface::native_mint::ID);

    // And a token account with 100 tokens.

    let owner = Keypair::new();

    let account =
        account::initialize(&mut context, &mint, &owner.pubkey(), &TOKEN_PROGRAM_ID).await;

    mint::mint(
        &mut context,
        &mint,
        &account,
        &mint_authority,
        100,
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    // When we burn 50 tokens.

    let burn_ix =
        spl_token::instruction::burn(&spl_token::ID, &account, &native_mint, &owner.pubkey(), &[], 50)
            .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[burn_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &owner],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::Custom(3)));
}
