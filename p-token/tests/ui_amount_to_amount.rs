#![cfg(feature = "test-sbf")]

mod setup;

use setup::{mint, TOKEN_PROGRAM_ID};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{pubkey::Pubkey, signature::Signer, transaction::Transaction};

#[test_case::test_case(TOKEN_PROGRAM_ID ; "p-token")]
#[tokio::test]
async fn ui_amount_to_amount(token_program: Pubkey) {
    let mut context = ProgramTest::new("token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();

    let mint = mint::initialize(
        &mut context,
        mint_authority,
        Some(freeze_authority),
        &token_program,
    )
    .await
    .unwrap();

    let mut ui_amount_to_amount_ix =
        spl_token::instruction::ui_amount_to_amount(&spl_token::ID, &mint, "1000.00").unwrap();
    // Switches the program id to the token program.
    ui_amount_to_amount_ix.program_id = token_program;

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
