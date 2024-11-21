use pinocchio::{account_info::AccountInfo, pubkey::Pubkey, ProgramResult};

use super::shared;

pub fn process_mint_to(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    shared::mint_to::process_mint_to(program_id, accounts, amount, None)
}
