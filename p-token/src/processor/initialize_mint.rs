use pinocchio::{account_info::AccountInfo, ProgramResult};

use super::shared::{self, initialize_mint::InitializeMint};

pub fn process_initialize_mint(accounts: &[AccountInfo], args: &InitializeMint) -> ProgramResult {
    shared::initialize_mint::process_initialize_mint(accounts, args, true)
}
