use pinocchio::{account_info::AccountInfo, pubkey::Pubkey, ProgramResult};

use super::shared;

#[inline(always)]
pub fn process_initialize_account2(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    owner: &Pubkey,
) -> ProgramResult {
    shared::initialize_account::process_initialize_account(program_id, accounts, Some(owner), true)
}
