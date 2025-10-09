mod setup;

use {
    crate::setup::mollusk::{create_mint_account, mollusk},
    core::str::from_utf8,
    mollusk_svm::result::Check,
    setup::{mint, TOKEN_PROGRAM_ID},
    solana_program_error::ProgramError,
    solana_program_test::{tokio, ProgramTest},
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::Transaction,
};

#[tokio::test]
async fn ui_amount_to_amount() {
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

    let ui_amount_to_amount_ix = spl_token_interface::instruction::ui_amount_to_amount(
        &spl_token_interface::ID,
        &mint,
        "1000.00",
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ui_amount_to_amount_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the transaction should succeed.

    let account = context.banks_client.get_account(mint).await.unwrap();

    assert!(account.is_some());
}

#[test]
fn ui_amount_to_amount_with_maximum_decimals() {
    // Given a mint account with `u8::MAX` as decimals.

    let mint = Pubkey::new_unique();
    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();

    let mint_account = create_mint_account(
        mint_authority,
        Some(freeze_authority),
        u8::MAX,
        &TOKEN_PROGRAM_ID,
    );

    // String representing the ui value `0.000....002`
    let mut ui_amount = [b'0'; u8::MAX as usize + 1];
    ui_amount[1] = b'.';
    ui_amount[ui_amount.len() - 1] = b'2';

    let input = from_utf8(&ui_amount).unwrap();

    // When we convert the ui amount using the mint, the transaction should
    // succeed and return 20 as the amount.

    let instruction = spl_token_interface::instruction::ui_amount_to_amount(
        &spl_token_interface::ID,
        &mint,
        input,
    )
    .unwrap();

    mollusk().process_and_validate_instruction(
        &instruction,
        &[(mint, mint_account)],
        &[Check::success(), Check::return_data(&20u64.to_le_bytes())],
    );
}

#[test]
fn fail_ui_amount_to_amount_with_invalid_ui_amount() {
    // Given a mint account with `u8::MAX` as decimals.

    let mint = Pubkey::new_unique();
    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();

    let mint_account = create_mint_account(
        mint_authority,
        Some(freeze_authority),
        u8::MAX,
        &TOKEN_PROGRAM_ID,
    );

    // String representing the ui value `2.0`
    let ui_amount = [b'2', b'.', b'0'];
    let input = from_utf8(&ui_amount).unwrap();

    // When we try to convert the ui amount using the mint, the transaction should
    // fail with an error since the resulting value does not fit in an `u64`.

    let instruction = spl_token_interface::instruction::ui_amount_to_amount(
        &spl_token_interface::ID,
        &mint,
        input,
    )
    .unwrap();

    mollusk().process_and_validate_instruction(
        &instruction,
        &[(mint, mint_account)],
        &[Check::err(ProgramError::InvalidArgument)],
    );
}
