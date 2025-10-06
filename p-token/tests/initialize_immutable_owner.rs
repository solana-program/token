mod setup;

use {
    setup::TOKEN_PROGRAM_ID,
    solana_keypair::Keypair,
    solana_program_pack::Pack,
    solana_program_test::{tokio, ProgramTest},
    solana_signer::Signer,
    solana_system_interface::instruction::create_account,
    solana_transaction::Transaction,
    spl_token_interface::state::AccountState,
};

#[tokio::test]
async fn initialize_immutable_owner() {
    let context = ProgramTest::new("pinocchio_token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given an uninitialize account.

    let account = Keypair::new();

    let account_size = 165;
    let rent = context.banks_client.get_rent().await.unwrap();

    // When we execute the initialize_immutable_owner instruction.

    let instructions = vec![
        create_account(
            &context.payer.pubkey(),
            &account.pubkey(),
            rent.minimum_balance(account_size),
            account_size as u64,
            &TOKEN_PROGRAM_ID,
        ),
        spl_token_interface::instruction::initialize_immutable_owner(
            &TOKEN_PROGRAM_ID,
            &account.pubkey(),
        )
        .unwrap(),
    ];

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&context.payer.pubkey()),
        &[&context.payer, &account],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the instruction should succeed.

    let account = context
        .banks_client
        .get_account(account.pubkey())
        .await
        .unwrap();

    assert!(account.is_some());

    let account = account.unwrap();
    let account = spl_token_interface::state::Account::unpack_unchecked(&account.data).unwrap();

    assert_eq!(account.state, AccountState::Uninitialized);
}
