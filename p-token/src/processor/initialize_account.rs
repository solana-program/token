use pinocchio::{account_info::AccountInfo, pubkey::Pubkey, ProgramResult};

use super::shared;

#[inline(never)]
pub fn process_initialize_account(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    shared::initialize_account::process_initialize_account(program_id, accounts, None, true)
}
