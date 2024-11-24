use pinocchio::{account_info::AccountInfo, ProgramResult};

use super::shared;

#[inline(always)]
pub fn process_initialize_account(accounts: &[AccountInfo]) -> ProgramResult {
    shared::initialize_account::process_initialize_account(accounts, None, true)
}
