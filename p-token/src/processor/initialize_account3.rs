use pinocchio::{account_info::AccountInfo, pubkey::Pubkey, ProgramResult};

use super::initialize_account::process_initialize_account;

#[inline(always)]
pub fn process_initialize_account3(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    owner: &Pubkey,
) -> ProgramResult {
    process_initialize_account(program_id, accounts, Some(owner), false)
}
