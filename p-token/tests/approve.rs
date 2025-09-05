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
async fn approve() {
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

    // When we approve a delegate.

    let delegate = Pubkey::new_unique();

    let approve_ix = spl_token::instruction::approve(
        &spl_token::ID,
        &account,
        &delegate,
        &owner.pubkey(),
        &[],
        50,
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[approve_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &owner],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the account should have the delegate and delegated amount.

    let account = context.banks_client.get_account(account).await.unwrap();

    assert!(account.is_some());

    let account = account.unwrap();
    let account = spl_token::state::Account::unpack(&account.data).unwrap();

    assert!(account.delegate.is_some());
    assert!(account.delegate.unwrap() == delegate);
    assert!(account.delegated_amount == 50);
}

#[tokio::test]
async fn approve_invalid_src() {
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

    // When we approve a delegate.

    let delegate = Pubkey::new_unique();

    let approve_ix = spl_token::instruction::approve(
        &spl_token::ID,
        &Pubkey::new_unique(), // Changed to invalid value
        &delegate,
        &owner.pubkey(),
        &[],
        50,
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[approve_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &owner],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::InvalidAccountData));
}

#[tokio::test]
async fn approve_frozen() {
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

    // Freeze account

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

    // When we approve a delegate.

    let delegate = Pubkey::new_unique();

    let approve_ix = spl_token::instruction::approve(
        &spl_token::ID,
        &account,
        &delegate,
        &owner.pubkey(),
        &[],
        50,
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[approve_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &owner],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::Custom(17)));
}
