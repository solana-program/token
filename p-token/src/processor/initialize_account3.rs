use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::PUBKEY_BYTES,
    ProgramResult,
};

use super::shared;

#[inline(always)]
pub fn process_initialize_account3(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // SAFETY: validate `instruction_data` length.
    // JC Nit: same here, can this be done without unsafe? Like this?
    if instruction_data.len() != PUBKEY_BYTES {
        return Err(ProgramError::InvalidInstructionData);
    }
    let owner = instruction_data.try_into().unwrap();
    shared::initialize_account::process_initialize_account(accounts, Some(owner), false)
}
