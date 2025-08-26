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
async fn mint_to() {
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

    // When we mint tokens to it.

    let mint_ix = spl_token::instruction::mint_to(
        &spl_token::ID,
        &mint,
        &account,
        &mint_authority.pubkey(),
        &[],
        100,
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[mint_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mint_authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then an account has the correct data.

    let account = context.banks_client.get_account(account).await.unwrap();

    assert!(account.is_some());

    let account = account.unwrap();
    let account = spl_token::state::Account::unpack(&account.data).unwrap();

    assert!(account.amount == 100);
}

#[tokio::test]
async fn mint_to_invalid_destination() {
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

    let mint_ix = spl_token::instruction::mint_to(
        &spl_token::ID,
        &mint,
        &Pubkey::new_unique(), // Changed to invalid Account
        &mint_authority.pubkey(),
        &[],
        100,
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[mint_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mint_authority],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::InvalidAccountData));
}

#[tokio::test]
async fn mint_to_frozen_account() {
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

    // And a token account.

    let owner = Keypair::new();

    let account =
        account::initialize(&mut context, &mint, &owner.pubkey(), &TOKEN_PROGRAM_ID).await;

    // When we mint tokens to it.

    let mint_ix = spl_token::instruction::mint_to(
        &spl_token::ID,
        &mint,
        &account,
        &mint_authority.pubkey(),
        &[],
        100,
    )
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

    let tx = Transaction::new_signed_with_payer(
        &[mint_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mint_authority],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::Custom(17)));
}

#[tokio::test]
async fn mint_to_native() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Keypair::new();

    let mint = Pubkey::from(pinocchio_token_interface::native_mint::ID);

    // And a token account.

    let owner = Keypair::new();

    let account =
        account::initialize(&mut context, &mint, &owner.pubkey(), &TOKEN_PROGRAM_ID).await;

    // When we mint tokens to it.

    let mint_ix = spl_token::instruction::mint_to(
        &spl_token::ID,
        &mint,
        &account,
        &mint_authority.pubkey(),
        &[],
        100,
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[mint_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mint_authority],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::Custom(10)));
}

#[tokio::test]
async fn mint_to_different_mint() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Keypair::new();
    let freeze_authority = Pubkey::new_unique();

    let mint1 = mint::initialize(
        &mut context,
        mint_authority.pubkey(),
        Some(freeze_authority),
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    let mint2 = mint::initialize(
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
        account::initialize(&mut context, &mint1, &owner.pubkey(), &TOKEN_PROGRAM_ID).await;

    // When we mint tokens to it.

    let mint_ix = spl_token::instruction::mint_to(
        &spl_token::ID,
        &mint2,
        &account,
        &mint_authority.pubkey(),
        &[],
        100,
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[mint_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mint_authority],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::Custom(3)));
}

#[tokio::test]
async fn mint_to_overflow() {
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

    // When we mint tokens to it.

    let mint_ix = spl_token::instruction::mint_to(
        &spl_token::ID,
        &mint,
        &account,
        &mint_authority.pubkey(),
        &[],
        100,
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[mint_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mint_authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let mint_excessive_ix = spl_token::instruction::mint_to(
        &spl_token::ID,
        &mint,
        &account,
        &mint_authority.pubkey(),
        &[],
        u64::MAX,
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[mint_excessive_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mint_authority],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::Custom(14)));
}

#[tokio::test]
async fn mint_to_zero() {
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

    // When we mint tokens to it.

    let amount = 0;

    let mint_ix = spl_token::instruction::mint_to(
        &spl_token::ID,
        &mint,
        &account,
        &mint_authority.pubkey(),
        &[],
        amount,
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[mint_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mint_authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then an account has the correct data.

    let account = context.banks_client.get_account(account).await.unwrap();

    assert!(account.is_some());

    let account = account.unwrap();
    let account = spl_token::state::Account::unpack(&account.data).unwrap();

    assert!(account.amount == 0);
}
