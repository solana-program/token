use pinocchio::{account_info::AccountInfo, ProgramResult};

use super::shared;

#[inline(always)]
pub fn process_initialize_multisig2(accounts: &[AccountInfo], m: u8) -> ProgramResult {
    shared::initialize_multisig::process_initialize_multisig(accounts, m, false)
}
