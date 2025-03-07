#![cfg(feature = "test-sbf")]

mod setup;

use std::mem::size_of;

use setup::TOKEN_PROGRAM_ID;
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use spl_token_interface::state::mint::Mint;

const EXCESS_LAMPORTS: u64 = 1_000_000_000;

#[test_case::test_case(TOKEN_PROGRAM_ID ; "p-token")]
#[tokio::test]
async fn withdraw_excess_lamports(token_program: Pubkey) {
    let context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint authority, freeze authority and an account keypair.

    let mint_authority = Keypair::new();
    let freeze_authority = Pubkey::new_unique();
    let account = Keypair::new();
    let account_pubkey = account.pubkey();

    let account_size = size_of::<Mint>();
    let rent = context.banks_client.get_rent().await.unwrap();

    let mut initialize_ix = spl_token::instruction::initialize_mint(
        &spl_token::ID,
        &account.pubkey(),
        &mint_authority.pubkey(),
        Some(&freeze_authority),
        0,
    )
    .unwrap();
    // Switches the program id to the token program.
    initialize_ix.program_id = token_program;

    // And we initialize a mint account with excess lamports.

    let instructions = vec![
        system_instruction::create_account(
            &context.payer.pubkey(),
            &account.pubkey(),
            rent.minimum_balance(account_size) + EXCESS_LAMPORTS,
            account_size as u64,
            &token_program,
        ),
        initialize_ix,
    ];

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&context.payer.pubkey()),
        &[&context.payer, &account],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let account = context
        .banks_client
        .get_account(account.pubkey())
        .await
        .unwrap();

    assert!(account.is_some());

    let account = account.unwrap();
    assert_eq!(
        account.lamports,
        rent.minimum_balance(account_size) + EXCESS_LAMPORTS
    );

    // When we withdraw the excess lamports.

    let destination = Pubkey::new_unique();

    let mut withdraw_ix = spl_token_2022::instruction::withdraw_excess_lamports(
        &spl_token_2022::ID,
        &account_pubkey,
        &destination,
        &mint_authority.pubkey(),
        &[],
    )
    .unwrap();
    // Switches the program id to the token program.
    withdraw_ix.program_id = token_program;

    let tx = Transaction::new_signed_with_payer(
        &[withdraw_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mint_authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let destination = context.banks_client.get_account(destination).await.unwrap();

    assert!(destination.is_some());

    let destination = destination.unwrap();
    assert_eq!(destination.lamports, EXCESS_LAMPORTS);
}
