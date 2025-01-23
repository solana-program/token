#![cfg(feature = "test-sbf")]

mod setup;

use setup::{mint, TOKEN_PROGRAM_ID};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    program_option::COption,
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_token::instruction::AuthorityType;

#[test_case::test_case(TOKEN_PROGRAM_ID ; "p-token")]
#[tokio::test]
async fn set_authority(token_program: Pubkey) {
    let mut context = ProgramTest::new("token_program", TOKEN_PROGRAM_ID, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Keypair::new();
    let freeze_authority = Keypair::new();

    let mint = mint::initialize(
        &mut context,
        mint_authority.pubkey(),
        Some(freeze_authority.pubkey()),
        &token_program,
    )
    .await
    .unwrap();

    // When we set a new freeze authority.

    let new_authority = Pubkey::new_unique();

    let mut set_authority_ix = spl_token::instruction::set_authority(
        &spl_token::ID,
        &mint,
        Some(&new_authority),
        AuthorityType::FreezeAccount,
        &freeze_authority.pubkey(),
        &[],
    )
    .unwrap();
    set_authority_ix.program_id = token_program;

    let tx = Transaction::new_signed_with_payer(
        &[set_authority_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &freeze_authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the account should have the delegate and delegated amount.

    let account = context.banks_client.get_account(mint).await.unwrap();

    assert!(account.is_some());

    let account = account.unwrap();
    let mint = spl_token::state::Mint::unpack(&account.data).unwrap();

    assert!(mint.freeze_authority == COption::Some(new_authority));
}
