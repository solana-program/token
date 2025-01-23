#![cfg(feature = "test-sbf")]

mod setup;

use std::mem::size_of;

use setup::TOKEN_PROGRAM_ID;
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    program_option::COption,
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use token_interface::state::mint::Mint;

#[test_case::test_case(TOKEN_PROGRAM_ID ; "p-token")]
#[tokio::test]
async fn initialize_mint(token_program: Pubkey) {
    let mut context = ProgramTest::new("token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint authority, freeze authority and an account keypair.

    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();
    let account = Keypair::new();

    let account_size = size_of::<Mint>();
    let rent = context.banks_client.get_rent().await.unwrap();

    let mut initialize_ix = spl_token::instruction::initialize_mint(
        &spl_token::ID,
        &account.pubkey(),
        &mint_authority,
        Some(&freeze_authority),
        0,
    )
    .unwrap();
    // Switches the program id to the token program.
    initialize_ix.program_id = token_program;

    // When a new mint account is created and initialized.

    let instructions = vec![
        system_instruction::create_account(
            &context.payer.pubkey(),
            &account.pubkey(),
            rent.minimum_balance(account_size),
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
