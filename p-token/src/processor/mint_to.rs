use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

use super::shared;

#[inline(always)]
pub fn process_mint_to(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let amount = u64::from_le_bytes(
        instruction_data
            .try_into()
            .map_err(|_error| ProgramError::InvalidInstructionData)?,
    );

    shared::mint_to::process_mint_to(accounts, amount, None)
}
