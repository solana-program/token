use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::{Pubkey, PUBKEY_BYTES},
    ProgramResult,
};

use super::shared;

#[inline(always)]
pub fn process_initialize_account2(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // SAFETY: validate `instruction_data` length.
    let owner = unsafe {
        if instruction_data.len() != PUBKEY_BYTES {
            return Err(ProgramError::InvalidInstructionData);
        } else {
            &*(instruction_data.as_ptr() as *const Pubkey)
        }
    };
    shared::initialize_account::process_initialize_account(accounts, Some(owner), true)
}
