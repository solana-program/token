use {
    pinocchio_token_interface::state::mint::Mint,
    solana_keypair::Keypair,
    solana_program_error::ProgramError,
    solana_program_test::{BanksClientError, ProgramTestContext},
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_system_interface::instruction::create_account,
    solana_transaction::Transaction,
    std::mem::size_of,
};

pub async fn initialize(
    context: &mut ProgramTestContext,
    mint_authority: Pubkey,
    freeze_authority: Option<Pubkey>,
    program_id: &Pubkey,
) -> Result<Pubkey, ProgramError> {
    initialize_with_decimals(context, mint_authority, freeze_authority, 4, program_id).await
}

pub async fn initialize_with_decimals(
    context: &mut ProgramTestContext,
    mint_authority: Pubkey,
    freeze_authority: Option<Pubkey>,
    decimals: u8,
    program_id: &Pubkey,
) -> Result<Pubkey, ProgramError> {
    // Mint account keypair.
    let account = Keypair::new();

    let account_size = size_of::<Mint>();
    let rent = context.banks_client.get_rent().await.unwrap();

    let mut initialize_ix = spl_token_interface::instruction::initialize_mint(
        &spl_token_interface::ID,
        &account.pubkey(),
        &mint_authority,
        freeze_authority.as_ref(),
        decimals,
    )
    .unwrap();
    // Switches the program id in case we are using a "custom" one.
    initialize_ix.program_id = *program_id;

    // Create a new account and initialize as a mint.

    let instructions = vec![
        create_account(
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

pub async fn mint(
    context: &mut ProgramTestContext,
    mint: &Pubkey,
    account: &Pubkey,
    mint_authority: &Keypair,
    amount: u64,
    program_id: &Pubkey,
) -> Result<(), BanksClientError> {
    let mut mint_ix = spl_token_interface::instruction::mint_to(
        &spl_token_interface::ID,
        mint,
        account,
        &mint_authority.pubkey(),
        &[],
        amount,
    )
    .unwrap();
    // Switches the program id to the token program.
    mint_ix.program_id = *program_id;

    let tx = Transaction::new_signed_with_payer(
        &[mint_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, mint_authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await
}
