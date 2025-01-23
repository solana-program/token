#![cfg(feature = "test-sbf")]

mod setup;

use setup::{account, mint, TOKEN_PROGRAM_ID};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_token::state::AccountState;

#[test_case::test_case(TOKEN_PROGRAM_ID ; "p-token")]
#[tokio::test]
async fn freeze_account(token_program: Pubkey) {
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

    // And a token account.

    let owner = Keypair::new();

    let account = account::initialize(&mut context, &mint, &owner.pubkey(), &token_program).await;

    let token_account = context.banks_client.get_account(account).await.unwrap();
    assert!(token_account.is_some());

    // When we freeze the account.

    let mut freeze_account_ix = spl_token::instruction::freeze_account(
        &spl_token::ID,
        &account,
        &mint,
        &freeze_authority.pubkey(),
        &[],
    )
    .unwrap();
    freeze_account_ix.program_id = token_program;

    let tx = Transaction::new_signed_with_payer(
        &[freeze_account_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &freeze_authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the account is frozen.

    let token_account = context.banks_client.get_account(account).await.unwrap();
    assert!(token_account.is_some());

    let token_account = token_account.unwrap();
    let token_account = spl_token::state::Account::unpack(&token_account.data).unwrap();

    assert_eq!(token_account.state, AccountState::Frozen);
}
