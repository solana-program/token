use pinocchio::{account_info::AccountInfo, ProgramResult};

use super::initialize_mint::process_initialize_mint;

#[inline(always)]
pub fn process_initialize_mint2(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    process_initialize_mint(accounts, instruction_data, false)
}
