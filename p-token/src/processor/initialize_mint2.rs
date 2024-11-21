use pinocchio::{account_info::AccountInfo, ProgramResult};

use super::shared::{self, initialize_mint::InitializeMint};

#[inline(always)]
pub fn process_initialize_mint2(accounts: &[AccountInfo], args: &InitializeMint) -> ProgramResult {
    shared::initialize_mint::process_initialize_mint(accounts, args, false)
}
