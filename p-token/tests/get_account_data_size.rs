mod setup;

use {
    setup::{mint, TOKEN_PROGRAM_ID},
    solana_program_test::{tokio, ProgramTest},
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::Transaction,
};

#[tokio::test]
async fn get_account_data_size() {
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

    let get_size_ix = spl_token::instruction::get_account_data_size(&TOKEN_PROGRAM_ID, &mint).unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[get_size_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;

    // Then the transaction should succeed.
    result.unwrap();

    // TODO: Figure out how to read return data
}
