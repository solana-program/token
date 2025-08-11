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
async fn transfer() {
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

    // When we transfer the tokens.

    let destination = Pubkey::new_unique();

    let destination_account =
        account::initialize(&mut context, &mint, &destination, &TOKEN_PROGRAM_ID).await;

    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::ID,
        &account,
        &destination_account,
        &owner.pubkey(),
        &[],
        100,
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[transfer_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &owner],
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

#[tokio::test]
async fn transfer_src_uninit() {
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

    // When we transfer the tokens.

    let destination = Pubkey::new_unique();

    let destination_account =
        account::initialize(&mut context, &mint, &destination, &TOKEN_PROGRAM_ID).await;

    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::ID,
        &Pubkey::new_unique(), // <-- Uninitialised Account
        &destination_account,
        &owner.pubkey(),
        &[],
        100,
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[transfer_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &owner],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::InvalidAccountData));
}

#[tokio::test]
async fn transfer_dst_uninit() {
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

    // When we transfer the tokens.

    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::ID,
        &account,
        &Pubkey::new_unique(), // <-- Uninitialised Account
        &owner.pubkey(),
        &[],
        100,
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[transfer_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &owner],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::InvalidAccountData));
}

#[tokio::test]
async fn transfer_frozen() {
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

    // When we transfer the tokens.

    let destination = Pubkey::new_unique();

    let destination_account =
        account::initialize(&mut context, &mint, &destination, &TOKEN_PROGRAM_ID).await;

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

    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::ID,
        &account,
        &destination_account,
        &owner.pubkey(),
        &[],
        100,
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[transfer_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &owner],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();

    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::Custom(17)));
}

#[tokio::test]
async fn transfer_frozen_self() {
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

    // When we transfer the tokens.

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

    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::ID,
        &account,
        &account,
        &owner.pubkey(),
        &[],
        100,
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[transfer_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &owner],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();

    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::Custom(17)));
}

#[tokio::test]
async fn transfer_insufficient() {
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

    // When we transfer the tokens.

    let destination = Pubkey::new_unique();

    let destination_account =
        account::initialize(&mut context, &mint, &destination, &TOKEN_PROGRAM_ID).await;

    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::ID,
        &account,
        &destination_account,
        &owner.pubkey(),
        &[],
        1000,
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[transfer_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &owner],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();

    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::Custom(1)));
}

#[tokio::test]
async fn transfer_insufficient_self() {
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

    // When we transfer the tokens.

    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::ID,
        &account,
        &account,
        &owner.pubkey(),
        &[],
        1000,
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[transfer_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &owner],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();

    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::Custom(1)));
}

#[tokio::test]
async fn transfer_diff_mint() {
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

    // And a token account with 100 tokens.

    let owner = Keypair::new();

    let account =
        account::initialize(&mut context, &mint1, &owner.pubkey(), &TOKEN_PROGRAM_ID).await;

    mint::mint(
        &mut context,
        &mint1,
        &account,
        &mint_authority,
        100,
        &TOKEN_PROGRAM_ID,
    )
    .await
    .unwrap();

    // When we transfer the tokens.

    let destination = Pubkey::new_unique();

    let destination_account =
        account::initialize(&mut context, &mint2, &destination, &TOKEN_PROGRAM_ID).await;

    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::ID,
        &account,
        &destination_account,
        &owner.pubkey(),
        &[],
        100,
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[transfer_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &owner],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::Custom(3)));
}

#[tokio::test]
async fn transfer_zero_amount() {
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

    // When we transfer the tokens.

    let destination = Pubkey::new_unique();

    let destination_account =
        account::initialize(&mut context, &mint, &destination, &TOKEN_PROGRAM_ID).await;

    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::ID,
        &account,
        &destination_account,
        &owner.pubkey(),
        &[],
        0,
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[transfer_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &owner],
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
