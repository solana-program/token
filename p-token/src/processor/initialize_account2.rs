use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

use super::shared;

#[inline(always)]
pub fn process_initialize_account2(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let owner: &Pubkey = instruction_data
        .try_into()
        .map_err(|_error| ProgramError::InvalidInstructionData)?;

    shared::initialize_account::process_initialize_account(accounts, Some(owner), true)
}
