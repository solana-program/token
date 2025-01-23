#![cfg(feature = "test-sbf")]

mod setup;

use setup::{account, mint, TOKEN_PROGRAM_ID};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

#[test_case::test_case(TOKEN_PROGRAM_ID ; "p-token")]
#[tokio::test]
async fn approve_checked(token_program: Pubkey) {
    let mut context = ProgramTest::new("token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Keypair::new();
    let freeze_authority = Pubkey::new_unique();

    let mint = mint::initialize(
        &mut context,
        mint_authority.pubkey(),
        Some(freeze_authority),
        &token_program,
    )
    .await
    .unwrap();

    // And a token account with 100 tokens.

    let owner = Keypair::new();

    let account = account::initialize(&mut context, &mint, &owner.pubkey(), &token_program).await;

    mint::mint(
        &mut context,
        &mint,
        &account,
        &mint_authority,
        100,
        &token_program,
    )
    .await
    .unwrap();

    // When we approve a delegate.

    let delegate = Pubkey::new_unique();

    let mut approve_ix = spl_token::instruction::approve_checked(
        &spl_token::ID,
        &account,
        &mint,
        &delegate,
        &owner.pubkey(),
        &[],
        50,
        4,
    )
    .unwrap();
    approve_ix.program_id = token_program;

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
