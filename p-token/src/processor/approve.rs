use pinocchio::{account_info::AccountInfo, pubkey::Pubkey, ProgramResult};

use super::shared;

pub fn process_approve(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    shared::approve::process_approve(program_id, accounts, amount, None)
}
