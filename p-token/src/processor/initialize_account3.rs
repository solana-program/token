use {
    super::shared,
    pinocchio::{
        account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
    },
};

#[inline(always)]
pub fn process_initialize_account3(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let owner: &Pubkey = instruction_data
        .try_into()
        .map_err(|_error| ProgramError::InvalidInstructionData)?;

    shared::initialize_account::process_initialize_account(accounts, Some(owner), false)
}
