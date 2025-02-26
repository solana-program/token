use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

use super::shared;

#[inline(always)]
pub fn process_transfer_checked(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // expected u64 (8) + u8 (1)
    let (amount, decimals) = if instruction_data.len() == 9 {
        let (amount, decimals) = instruction_data.split_at(core::mem::size_of::<u64>());
        // JC: feel free to unwrap since you checked the length earlier!
        (
            u64::from_le_bytes(
                amount
                    .try_into()
                    .map_err(|_error| ProgramError::InvalidInstructionData)?,
            ),
            decimals.first(),
        )
    } else {
        return Err(ProgramError::InvalidInstructionData);
    };

    shared::transfer::process_transfer(accounts, amount, decimals.copied())
}
