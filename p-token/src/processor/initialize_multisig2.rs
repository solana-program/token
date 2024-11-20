use pinocchio::{account_info::AccountInfo, ProgramResult};

use super::initialize_multisig::process_initialize_multisig;

pub fn process_initialize_multisig2(accounts: &[AccountInfo], m: u8) -> ProgramResult {
    process_initialize_multisig(accounts, m, false)
}
