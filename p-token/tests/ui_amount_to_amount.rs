mod setup;

use {
    setup::{mint, TOKEN_PROGRAM_ID},
    solana_program_test::{tokio, ProgramTest},
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::Transaction,
};

#[tokio::test]
async fn ui_amount_to_amount() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();

    let mint = mint::initialize(
        &mut context,
        mint_authority,
        Some(freeze_authority),
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    let ui_amount_to_amount_ix =
        spl_token::instruction::ui_amount_to_amount(&spl_token::ID, &mint, "1000.00").unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ui_amount_to_amount_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the transaction should succeed.

    let account = context.banks_client.get_account(mint).await.unwrap();

    assert!(account.is_some());
}

#[tokio::test]
async fn ui_amount_to_amount_empty() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();

    let mint = mint::initialize(
        &mut context,
        mint_authority,
        Some(freeze_authority),
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    let ui_amount_to_amount_ix =
        spl_token::instruction::ui_amount_to_amount(&spl_token::ID, &mint, "").unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ui_amount_to_amount_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::InvalidArgument));

}

#[tokio::test]
async fn ui_amount_to_amount_decimal_point_only() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();

    let mint = mint::initialize(
        &mut context,
        mint_authority,
        Some(freeze_authority),
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    let ui_amount_to_amount_ix =
        spl_token::instruction::ui_amount_to_amount(&spl_token::ID, &mint, ".").unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ui_amount_to_amount_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::InvalidArgument));

}

#[tokio::test]
async fn ui_amount_to_amount_decimals_empty_ints() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();

    let mint = mint::initialize(
        &mut context,
        mint_authority,
        Some(freeze_authority),
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    let ui_amount_to_amount_ix =
        spl_token::instruction::ui_amount_to_amount(&spl_token::ID, &mint, ".01").unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ui_amount_to_amount_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the transaction should succeed.

    let account = context.banks_client.get_account(mint).await.unwrap();

    assert!(account.is_some());
}

#[tokio::test]
async fn ui_amount_to_amount_decimals_empty_ints_zero() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();

    let mint = mint::initialize(
        &mut context,
        mint_authority,
        Some(freeze_authority),
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    let ui_amount_to_amount_ix =
        spl_token::instruction::ui_amount_to_amount(&spl_token::ID, &mint, ".0").unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ui_amount_to_amount_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::InvalidArgument));
}

#[tokio::test]
async fn ui_amount_to_amount_decimals_empty_fractions() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();

    let mint = mint::initialize(
        &mut context,
        mint_authority,
        Some(freeze_authority),
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    let ui_amount_to_amount_ix =
        spl_token::instruction::ui_amount_to_amount(&spl_token::ID, &mint, "1000.").unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ui_amount_to_amount_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the transaction should succeed.

    let account = context.banks_client.get_account(mint).await.unwrap();

    assert!(account.is_some());
}

#[tokio::test]
async fn ui_amount_to_amount_decimals_empty_fractions_zero() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();

    let mint = mint::initialize(
        &mut context,
        mint_authority,
        Some(freeze_authority),
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    let ui_amount_to_amount_ix =
        spl_token::instruction::ui_amount_to_amount(&spl_token::ID, &mint, "000.").unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ui_amount_to_amount_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the transaction should succeed.

    let account = context.banks_client.get_account(mint).await.unwrap();

    assert!(account.is_some());
}

#[tokio::test]
async fn ui_amount_to_amount_decimals_double() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();

    let mint = mint::initialize(
        &mut context,
        mint_authority,
        Some(freeze_authority),
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    let ui_amount_to_amount_ix =
        spl_token::instruction::ui_amount_to_amount(&spl_token::ID, &mint, "1000.0.0").unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ui_amount_to_amount_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::InvalidArgument));
}

#[tokio::test]
async fn ui_amount_to_amount_decimals_excessive_fractions() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();

    let mint = mint::initialize(
        &mut context,
        mint_authority,
        Some(freeze_authority),
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    let ui_amount_to_amount_ix =
        spl_token::instruction::ui_amount_to_amount(&spl_token::ID, &mint, ".11111").unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ui_amount_to_amount_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::InvalidArgument));
}

#[tokio::test]
async fn ui_amount_to_amount_decimals_excessive_ints() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();

    let mint = mint::initialize(
        &mut context,
        mint_authority,
        Some(freeze_authority),
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    let ui_amount_to_amount_ix =
        spl_token::instruction::ui_amount_to_amount(&spl_token::ID, &mint, "11111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111.0").unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ui_amount_to_amount_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::InvalidArgument));
}

#[tokio::test]
async fn ui_amount_to_amount_excessive_ints() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();

    let mint = mint::initialize(
        &mut context,
        mint_authority,
        Some(freeze_authority),
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    let ui_amount_to_amount_ix =
        spl_token::instruction::ui_amount_to_amount(&spl_token::ID, &mint, "11111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111").unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ui_amount_to_amount_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::InvalidArgument));
}

#[tokio::test]
async fn ui_amount_to_amount_plus_symbol() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();

    let mint = mint::initialize(
        &mut context,
        mint_authority,
        Some(freeze_authority),
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    let ui_amount_to_amount_ix =
        spl_token::instruction::ui_amount_to_amount(&spl_token::ID, &mint, "+100").unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ui_amount_to_amount_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the transaction should succeed.

    let account = context.banks_client.get_account(mint).await.unwrap();

    assert!(account.is_some());
}

#[tokio::test]
async fn ui_amount_to_amount_decimal_plus_symbol() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();

    let mint = mint::initialize(
        &mut context,
        mint_authority,
        Some(freeze_authority),
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    let ui_amount_to_amount_ix =
        spl_token::instruction::ui_amount_to_amount(&spl_token::ID, &mint, "+.01").unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ui_amount_to_amount_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the transaction should succeed.

    let account = context.banks_client.get_account(mint).await.unwrap();

    assert!(account.is_some());
}

// TODO: Why is this allowed?
#[tokio::test]
async fn ui_amount_to_amount_decimal_plus_only() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();

    let mint = mint::initialize(
        &mut context,
        mint_authority,
        Some(freeze_authority),
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    let ui_amount_to_amount_ix =
        spl_token::instruction::ui_amount_to_amount(&spl_token::ID, &mint, "+.").unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ui_amount_to_amount_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the transaction should succeed.

    let account = context.banks_client.get_account(mint).await.unwrap();

    assert!(account.is_some());
}

// TODO: Why is this allowed?
#[tokio::test]
async fn ui_amount_to_amount_plus_only() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();

    let mint = mint::initialize(
        &mut context,
        mint_authority,
        Some(freeze_authority),
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    let ui_amount_to_amount_ix =
        spl_token::instruction::ui_amount_to_amount(&spl_token::ID, &mint, "+").unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ui_amount_to_amount_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the transaction should succeed.

    let account = context.banks_client.get_account(mint).await.unwrap();

    assert!(account.is_some());
}

#[tokio::test]
async fn ui_amount_to_amount_minus_only() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();

    let mint = mint::initialize(
        &mut context,
        mint_authority,
        Some(freeze_authority),
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    let ui_amount_to_amount_ix =
        spl_token::instruction::ui_amount_to_amount(&spl_token::ID, &mint, "-").unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ui_amount_to_amount_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::InvalidArgument));
}

#[tokio::test]
async fn ui_amount_to_amount_decimal_minus_zero() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();

    let mint = mint::initialize(
        &mut context,
        mint_authority,
        Some(freeze_authority),
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    let ui_amount_to_amount_ix =
        spl_token::instruction::ui_amount_to_amount(&spl_token::ID, &mint, "-0").unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ui_amount_to_amount_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::InvalidArgument));
}

#[tokio::test]
async fn ui_amount_to_amount_decimal_minus_only() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();

    let mint = mint::initialize(
        &mut context,
        mint_authority,
        Some(freeze_authority),
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    let ui_amount_to_amount_ix =
        spl_token::instruction::ui_amount_to_amount(&spl_token::ID, &mint, "-.").unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ui_amount_to_amount_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::InvalidArgument));
}

#[tokio::test]
async fn ui_amount_to_amount_invalid_char() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();

    let mint = mint::initialize(
        &mut context,
        mint_authority,
        Some(freeze_authority),
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    let ui_amount_to_amount_ix =
        spl_token::instruction::ui_amount_to_amount(&spl_token::ID, &mint, "10a").unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ui_amount_to_amount_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::InvalidArgument));
}

#[tokio::test]
async fn ui_amount_to_amount_decimal_invalid_char() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();

    let mint = mint::initialize(
        &mut context,
        mint_authority,
        Some(freeze_authority),
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    let ui_amount_to_amount_ix =
        spl_token::instruction::ui_amount_to_amount(&spl_token::ID, &mint, "+10.a").unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ui_amount_to_amount_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::InvalidArgument));
}

// TODO: Why is 1844674407370955 the limit?
#[tokio::test]
async fn ui_amount_to_amount_max() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();

    let mint = mint::initialize(
        &mut context,
        mint_authority,
        Some(freeze_authority),
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    let ui_amount_to_amount_ix =
        spl_token::instruction::ui_amount_to_amount(&spl_token::ID, &mint, "1844674407370955").unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ui_amount_to_amount_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the transaction should succeed.

    let account = context.banks_client.get_account(mint).await.unwrap();

    assert!(account.is_some());
}

// TODO: Why is 1844674407370955 the limit?
#[tokio::test]
async fn ui_amount_to_amount_execessive_magnitude() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();

    let mint = mint::initialize(
        &mut context,
        mint_authority,
        Some(freeze_authority),
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    let ui_amount_to_amount_ix =
        spl_token::instruction::ui_amount_to_amount(&spl_token::ID, &mint, "1844674407370956").unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ui_amount_to_amount_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::InvalidArgument));
}

// TODO: Why is 1844674407370955 the limit?
#[tokio::test]
async fn ui_amount_to_amount_decimal_max() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();

    let mint = mint::initialize(
        &mut context,
        mint_authority,
        Some(freeze_authority),
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    let ui_amount_to_amount_ix =
        spl_token::instruction::ui_amount_to_amount(&spl_token::ID, &mint, "18446744073709.55").unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ui_amount_to_amount_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the transaction should succeed.

    let account = context.banks_client.get_account(mint).await.unwrap();

    assert!(account.is_some());
}

// TODO: Why is 1844674407370955.1 the limit?
#[tokio::test]
async fn ui_amount_to_amount_decimal_execessive_magnitude() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();

    let mint = mint::initialize(
        &mut context,
        mint_authority,
        Some(freeze_authority),
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    let ui_amount_to_amount_ix =
        spl_token::instruction::ui_amount_to_amount(&spl_token::ID, &mint, "1844674407370955.1").unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ui_amount_to_amount_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the transaction should succeed.

    let account = context.banks_client.get_account(mint).await.unwrap();

    assert!(account.is_some());
}

// TODO: Why is 1844674407370955.1 the limit?
#[tokio::test]
async fn ui_amount_to_amount_decimal_execessive_magnitude1() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();

    let mint = mint::initialize(
        &mut context,
        mint_authority,
        Some(freeze_authority),
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    let ui_amount_to_amount_ix =
        spl_token::instruction::ui_amount_to_amount(&spl_token::ID, &mint, "1844674407370955.2").unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ui_amount_to_amount_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::InvalidArgument));
}
