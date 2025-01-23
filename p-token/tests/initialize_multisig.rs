#![cfg(feature = "test-sbf")]

mod setup;

use setup::TOKEN_PROGRAM_ID;
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use spl_token::state::Multisig;

#[test_case::test_case(TOKEN_PROGRAM_ID ; "p-token")]
#[tokio::test]
async fn initialize_multisig(token_program: Pubkey) {
    let mut context = ProgramTest::new("token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given an account

    let multisig = Keypair::new();
    let signer1 = Pubkey::new_unique();
    let signer2 = Pubkey::new_unique();
    let signer3 = Pubkey::new_unique();
    let signers = vec![&signer1, &signer2, &signer3];

    let rent = context.banks_client.get_rent().await.unwrap();

    let mut initialize_ix = spl_token::instruction::initialize_multisig(
        &spl_token::ID,
        &multisig.pubkey(),
        &signers,
        2,
    )
    .unwrap();
    // Switches the program id to the token program.
    initialize_ix.program_id = token_program;

    // When a new multisig account is created and initialized.

    let instructions = vec![
        system_instruction::create_account(
            &context.payer.pubkey(),
            &multisig.pubkey(),
            rent.minimum_balance(Multisig::LEN),
            Multisig::LEN as u64,
            &token_program,
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
    let multisig = spl_token::state::Multisig::unpack(&account.data).unwrap();

    assert!(multisig.is_initialized);
    assert_eq!(multisig.n, 3);
    assert_eq!(multisig.m, 2);
}
