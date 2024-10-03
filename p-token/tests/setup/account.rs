use solana_program_test::ProgramTestContext;
use solana_sdk::{
    program_error::ProgramError, pubkey::Pubkey, signature::Keypair, signer::Signer,
    system_instruction, transaction::Transaction,
};

pub async fn initialize(
    context: &mut ProgramTestContext,
    mint: &Pubkey,
    owner: &Pubkey,
    program_id: &Pubkey,
) -> Result<Pubkey, ProgramError> {
    let account = Keypair::new();

    let account_size = 165;
    let rent = context.banks_client.get_rent().await.unwrap();

    let mut initialize_ix =
        spl_token::instruction::initialize_account(&spl_token::ID, &account.pubkey(), mint, owner)
            .unwrap();
    initialize_ix.program_id = *program_id;

    let instructions = vec![
        system_instruction::create_account(
            &context.payer.pubkey(),
            &account.pubkey(),
            rent.minimum_balance(account_size),
            account_size as u64,
            program_id,
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

    Ok(account.pubkey())
}
