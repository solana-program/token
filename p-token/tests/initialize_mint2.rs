mod setup;

use {
    setup::TOKEN_PROGRAM_ID,
    solana_keypair::Keypair,
    solana_program_option::COption,
    solana_program_pack::Pack,
    solana_program_test::{tokio, ProgramTest},
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_system_interface::instruction::create_account,
    solana_transaction::Transaction,
    spl_token_interface::state::mint::Mint,
    std::mem::size_of,
};

#[tokio::test]
async fn initialize_mint2() {
    let context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint authority, freeze authority and an account keypair.

    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();
    let account = Keypair::new();

    let account_size = size_of::<Mint>();
    let rent = context.banks_client.get_rent().await.unwrap();

    let initialize_ix = spl_token::instruction::initialize_mint2(
        &spl_token::ID,
        &account.pubkey(),
        &mint_authority,
        Some(&freeze_authority),
        0,
    )
    .unwrap();

    // When a new mint account is created and initialized.

    let instructions = vec![
        create_account(
            &context.payer.pubkey(),
            &account.pubkey(),
            rent.minimum_balance(account_size),
            account_size as u64,
            &TOKEN_PROGRAM_ID,
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

    // Then an account has the correct data.

    let account = context
        .banks_client
        .get_account(account.pubkey())
        .await
        .unwrap();

    assert!(account.is_some());

    let account = account.unwrap();
    let mint = spl_token::state::Mint::unpack(&account.data).unwrap();

    assert!(mint.is_initialized);
    assert!(mint.mint_authority == COption::Some(mint_authority));
    assert!(mint.freeze_authority == COption::Some(freeze_authority));
    assert!(mint.decimals == 0)
}
