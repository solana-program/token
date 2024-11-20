use pinocchio::{account_info::AccountInfo, ProgramResult};

use super::initialize_mint::{process_initialize_mint, InitializeMint};

pub fn process_initialize_mint2(accounts: &[AccountInfo], args: &InitializeMint) -> ProgramResult {
    process_initialize_mint(accounts, args, false)
}
