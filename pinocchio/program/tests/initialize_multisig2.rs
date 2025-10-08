mod setup;

use {
    setup::TOKEN_PROGRAM_ID,
    solana_keypair::Keypair,
    solana_program_pack::Pack,
    solana_program_test::{tokio, ProgramTest},
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_system_interface::instruction::create_account,
    solana_transaction::Transaction,
    spl_token_interface::state::Multisig,
};

#[tokio::test]
async fn initialize_multisig2() {
    let context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given an account

    let multisig = Keypair::new();
    let signer1 = Pubkey::new_unique();
    let signer2 = Pubkey::new_unique();
    let signer3 = Pubkey::new_unique();
    let signers = vec![&signer1, &signer2, &signer3];

    let rent = context.banks_client.get_rent().await.unwrap();

    let initialize_ix = spl_token_interface::instruction::initialize_multisig2(
        &spl_token_interface::ID,
        &multisig.pubkey(),
        &signers,
        2,
    )
    .unwrap();

    // When a new multisig account is created and initialized.

    let instructions = vec![
        create_account(
            &context.payer.pubkey(),
            &multisig.pubkey(),
            rent.minimum_balance(Multisig::LEN),
            Multisig::LEN as u64,
            &TOKEN_PROGRAM_ID,
        ),
        initialize_ix,
    ];

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&context.payer.pubkey()),
        &[&context.payer, &multisig],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the multisig has the correct data.

    let account = context
        .banks_client
        .get_account(multisig.pubkey())
        .await
        .unwrap();

    assert!(account.is_some());

    let account = account.unwrap();
    let multisig = spl_token_interface::state::Multisig::unpack(&account.data).unwrap();

    assert!(multisig.is_initialized);
    assert_eq!(multisig.n, 3);
    assert_eq!(multisig.m, 2);
}
