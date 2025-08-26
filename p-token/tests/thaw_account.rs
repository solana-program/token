mod setup;

use {
    setup::{account, mint, TOKEN_PROGRAM_ID},
    solana_keypair::Keypair,
    solana_program_pack::Pack,
    solana_program_test::{tokio, ProgramTest},
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::Transaction,
    spl_token::state::AccountState,
};

#[tokio::test]
async fn thaw_account() {
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

    // And a frozen token account.

    let owner = Keypair::new();

    let account =
        account::initialize(&mut context, &mint, &owner.pubkey(), &TOKEN_PROGRAM_ID).await;

    let token_account = context.banks_client.get_account(account).await.unwrap();
    assert!(token_account.is_some());

    account::freeze(
        &mut context,
        &account,
        &mint,
        &freeze_authority,
        &TOKEN_PROGRAM_ID,
    )
    .await;

    // When we thaw the account.

    let thaw_account_ix = spl_token::instruction::thaw_account(
        &spl_token::ID,
        &account,
        &mint,
        &freeze_authority.pubkey(),
        &[],
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[thaw_account_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &freeze_authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the account is frozen.

    let token_account = context.banks_client.get_account(account).await.unwrap();
    assert!(token_account.is_some());

    let token_account = token_account.unwrap();
    let token_account = spl_token::state::Account::unpack(&token_account.data).unwrap();

    assert_eq!(token_account.state, AccountState::Initialized);
}

#[tokio::test]
async fn thaw_account_invalid_src() {
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

    // And a frozen token account.

    let owner = Keypair::new();

    let account =
        account::initialize(&mut context, &mint, &owner.pubkey(), &TOKEN_PROGRAM_ID).await;

    let token_account = context.banks_client.get_account(account).await.unwrap();
    assert!(token_account.is_some());

    account::freeze(
        &mut context,
        &account,
        &mint,
        &freeze_authority,
        &TOKEN_PROGRAM_ID,
    )
    .await;

    // When we thaw the account.

    let thaw_account_ix = spl_token::instruction::thaw_account(
        &spl_token::ID,
        &Pubkey::new_unique(),
        &mint,
        &freeze_authority.pubkey(),
        &[],
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[thaw_account_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &freeze_authority],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::InvalidAccountData));
}

// TODO: Why does this not fail?
// #[tokio::test]
// async fn thaw_account_double_thaw() {
//     let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
//         .start_with_context()
//         .await;

//     // Given a mint account.

//     let mint_authority = Keypair::new();
//     let freeze_authority = Keypair::new();

//     let mint = mint::initialize(
//         &mut context,
//         mint_authority.pubkey(),
//         Some(freeze_authority.pubkey()),
//         &TOKEN_PROGRAM_ID,
//     )
//     .await
//     .unwrap();

//     // And a frozen token account.

//     let owner = Keypair::new();

//     let account =
//         account::initialize(&mut context, &mint, &owner.pubkey(), &TOKEN_PROGRAM_ID).await;

//     let token_account = context.banks_client.get_account(account).await.unwrap();
//     assert!(token_account.is_some());

//     account::freeze(
//         &mut context,
//         &account,
//         &mint,
//         &freeze_authority,
//         &TOKEN_PROGRAM_ID,
//     )
//     .await;

//     // When we thaw the account.

//     let thaw_account_ix = spl_token::instruction::thaw_account(
//         &spl_token::ID,
//         &account,
//         &mint,
//         &freeze_authority.pubkey(),
//         &[],
//     )
//     .unwrap();

//     let tx = Transaction::new_signed_with_payer(
//         &[thaw_account_ix],
//         Some(&context.payer.pubkey()),
//         &[&context.payer, &freeze_authority],
//         context.last_blockhash,
//     );
//     context.banks_client.process_transaction(tx).await.unwrap();

//     let token_account = context.banks_client.get_account(account).await.unwrap();
//     assert!(token_account.is_some());

//     let token_account = token_account.unwrap();
//     let token_account = spl_token::state::Account::unpack(&token_account.data).unwrap();

//     assert_eq!(token_account.state, AccountState::Initialized);

//     // Double thaw

//     let thaw_account_ix = spl_token::instruction::thaw_account(
//         &spl_token::ID,
//         &account,
//         &mint,
//         &freeze_authority.pubkey(),
//         &[],
//     )
//     .unwrap();

//     let tx = Transaction::new_signed_with_payer(
//         &[thaw_account_ix],
//         Some(&context.payer.pubkey()),
//         &[&context.payer, &freeze_authority],
//         context.last_blockhash,
//     );
//     let result = context.banks_client.process_transaction(tx).await;
//     let inner_error = result.err().unwrap().unwrap();
//     assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::Custom(13)));
// }

#[tokio::test]
async fn thaw_account_already_thawed() {
    let mut context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    let mint = Pubkey::from(pinocchio_token_interface::native_mint::ID);

    let owner = Keypair::new();

    let account =
        account::initialize(&mut context, &mint, &owner.pubkey(), &TOKEN_PROGRAM_ID).await;

    let token_account = context.banks_client.get_account(account).await.unwrap();
    assert!(token_account.is_some());
    let token_account = token_account.unwrap();
    let token_account = spl_token::state::Account::unpack(&token_account.data).unwrap();

    assert_eq!(token_account.state, AccountState::Initialized); // account is Initialized (not Frozen)

    // When we thaw the account.

    let freeze_authority = Keypair::new();
    let thaw_account_ix = spl_token::instruction::thaw_account(
        &spl_token::ID,
        &account,
        &mint,
        &freeze_authority.pubkey(),
        &[],
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[thaw_account_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &freeze_authority],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::Custom(13)));
}

#[tokio::test]
async fn thaw_account_mint_mismatch() {
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

    // And a frozen token account.

    let owner = Keypair::new();

    let account =
        account::initialize(&mut context, &mint, &owner.pubkey(), &TOKEN_PROGRAM_ID).await;

    let token_account = context.banks_client.get_account(account).await.unwrap();
    assert!(token_account.is_some());

    account::freeze(
        &mut context,
        &account,
        &mint,
        &freeze_authority,
        &TOKEN_PROGRAM_ID,
    )
    .await;

    // When we thaw the account.

    let thaw_account_ix = spl_token::instruction::thaw_account(
        &spl_token::ID,
        &account,
        &Pubkey::from(pinocchio_token_interface::native_mint::ID),
        &freeze_authority.pubkey(),
        &[],
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[thaw_account_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &freeze_authority],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;
    let inner_error = result.err().unwrap().unwrap();
    assert_eq!(inner_error, solana_transaction_error::TransactionError::InstructionError(0, solana_instruction::error::InstructionError::Custom(3)));
}
