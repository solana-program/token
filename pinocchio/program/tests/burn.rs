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
async fn burn() {
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

    // When we burn 50 tokens.

    let burn_ix = spl_token_interface::instruction::burn(
        &spl_token_interface::ID,
        &account,
        &mint,
        &owner.pubkey(),
        &[],
        50,
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[burn_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &owner],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the account should have 50 tokens remaining.

    let account = context.banks_client.get_account(account).await.unwrap();

    assert!(account.is_some());

    let account = account.unwrap();
    let account = spl_token_interface::state::Account::unpack(&account.data).unwrap();

    assert!(account.amount == 50);
}
