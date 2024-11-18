#![cfg(feature = "test-sbf")]

mod setup;

use setup::{account, mint};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

#[test_case::test_case(spl_token::ID ; "spl-token")]
#[test_case::test_case(Pubkey::new_from_array(token_program::ID) ; "p-token")]
#[tokio::test]
async fn revoke(token_program: Pubkey) {
    let program_id = Pubkey::new_from_array(token_program::ID);
    let mut context = ProgramTest::new("token_program", program_id, None)
        .start_with_context()
        .await;

    // Given a mint account.

    let mint_authority = Keypair::new();
    let freeze_authority = Pubkey::new_unique();

    let mint = mint::initialize(
        &mut context,
        mint_authority.pubkey(),
        Some(freeze_authority),
        &token_program,
    )
    .await
    .unwrap();

    // And a token account with 100 tokens.

    let owner = Keypair::new();

    let account = account::initialize(&mut context, &mint, &owner.pubkey(), &token_program).await;

    mint::mint(
        &mut context,
        &mint,
        &account,
        &mint_authority,
        100,
        &token_program,
    )
    .await
    .unwrap();

    // And 50 tokens delegated.

    let delegate = Pubkey::new_unique();

    account::approve(
        &mut context,
        &account,
        &delegate,
        &owner,
        50,
        &token_program,
    )
    .await;

    // When we revoke the delegation.

    let mut revoke_ix =
        spl_token::instruction::revoke(&spl_token::ID, &account, &owner.pubkey(), &[]).unwrap();
    revoke_ix.program_id = token_program;

    let tx = Transaction::new_signed_with_payer(
        &[revoke_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &owner],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the account should not have a delegate nor delegated amount.

    let account = context.banks_client.get_account(account).await.unwrap();

    assert!(account.is_some());

    let account = account.unwrap();
    let account = spl_token::state::Account::unpack(&account.data).unwrap();

    assert!(account.delegate.is_none());
    assert!(account.delegated_amount == 0);
}
