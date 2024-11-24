use pinocchio::{account_info::AccountInfo, pubkey::Pubkey, ProgramResult};

use super::shared;

#[inline(always)]
pub fn process_initialize_account2(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let owner = unsafe { &*(instruction_data.as_ptr() as *const Pubkey) };
    shared::initialize_account::process_initialize_account(accounts, Some(owner), true)
}
