mod setup;

use {
    setup::{account, mint, TOKEN_PROGRAM_ID},
    solana_keypair::Keypair,
    solana_program_test::{tokio, ProgramTest},
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::Transaction,
};

#[tokio::test]
async fn close_account() {
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

    // And a token account.

    let owner = Keypair::new();

    let account =
        account::initialize(&mut context, &mint, &owner.pubkey(), &TOKEN_PROGRAM_ID).await;

    let token_account = context.banks_client.get_account(account).await.unwrap();
    assert!(token_account.is_some());

    // When we close the account.

    let close_account_ix = spl_token::instruction::close_account(
        &spl_token::ID,
        &account,
        &owner.pubkey(),
        &owner.pubkey(),
        &[],
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[close_account_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &owner],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then an account must not exist.

    let token_account = context.banks_client.get_account(account).await.unwrap();
    assert!(token_account.is_none());
}

#[tokio::test]
async fn close_same_accounts() {
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

    // And a token account.

    let owner = Keypair::new();

    let account =
        account::initialize(&mut context, &mint, &owner.pubkey(), &TOKEN_PROGRAM_ID).await;

    let token_account = context.banks_client.get_account(account).await.unwrap();
    assert!(token_account.is_some());

    // When we close the account.

    let close_account_ix = spl_token::instruction::close_account(
        &spl_token::ID,
        &owner.pubkey(),
        &owner.pubkey(),
        &owner.pubkey(),
        &[],
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[close_account_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &owner],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::InvalidAccountData));
}

#[tokio::test]
async fn close_invalid_source() {
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

    // And a token account.

    let owner = Keypair::new();

    let account =
        account::initialize(&mut context, &mint, &owner.pubkey(), &TOKEN_PROGRAM_ID).await;

    let token_account = context.banks_client.get_account(account).await.unwrap();
    assert!(token_account.is_some());

    // When we close the account.

    let close_account_ix = spl_token::instruction::close_account(
        &spl_token::ID,
        &Pubkey::new_unique(),
        &owner.pubkey(),
        &owner.pubkey(),
        &[],
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[close_account_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &owner],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::InvalidAccountData));
}

#[tokio::test]
async fn close_non_native_with_balance() {
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

    // And a token account.

    let owner = Keypair::new();

    let account =
        account::initialize(&mut context, &mint, &owner.pubkey(), &TOKEN_PROGRAM_ID).await;

    let token_account = context.banks_client.get_account(account).await.unwrap();
    assert!(token_account.is_some());

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

    // When we close the account.

    let close_account_ix = spl_token::instruction::close_account(
        &spl_token::ID,
        &account,
        &owner.pubkey(),
        &owner.pubkey(),
        &[],
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[close_account_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &owner],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::Custom(11)));
}
