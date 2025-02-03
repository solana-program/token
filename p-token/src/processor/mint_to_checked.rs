use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

use super::shared;

#[inline(always)]
pub fn process_mint_to_checked(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let (amount, decimals) = instruction_data.split_at(core::mem::size_of::<u64>());

    let amount = u64::from_le_bytes(
        amount
            .try_into()
            .map_err(|_error| ProgramError::InvalidInstructionData)?,
    );

    shared::mint_to::process_mint_to(
        accounts,
        amount,
        Some(*decimals.first().ok_or(ProgramError::InvalidAccountData)?),
    )
}
