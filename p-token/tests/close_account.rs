#![cfg(feature = "test-sbf")]

mod setup;

use setup::{account, mint};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

#[test_case::test_case(spl_token::ID ; "spl-token")]
#[test_case::test_case(Pubkey::new_from_array(token_program::ID) ; "p-token")]
#[tokio::test]
async fn close_account(token_program: Pubkey) {
    let program_id = Pubkey::new_from_array(token_program::ID);
    let mut context = ProgramTest::new("token_program", program_id, None)
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

    // And a token account.

    let owner = Keypair::new();

    let account = account::initialize(&mut context, &mint, &owner.pubkey(), &token_program).await;

    let token_account = context.banks_client.get_account(account).await.unwrap();
    assert!(token_account.is_some());

    // When we close the account.

    let mut close_account_ix = spl_token::instruction::close_account(
        &spl_token::ID,
        &account,
        &owner.pubkey(),
        &owner.pubkey(),
        &[],
    )
    .unwrap();
    close_account_ix.program_id = token_program;

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
