use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

use super::shared;

#[inline(always)]
pub fn process_burn_checked(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    // expected u64 (8) + u8 (1)
    let (amount, decimals) = if instruction_data.len() == 9 {
        let (amount, decimals) = instruction_data.split_at(core::mem::size_of::<u64>());
        (
            u64::from_le_bytes(
                // JC: this should be unwrappabable since the length was just checked
                // and the split amount is valid -- if it gives CU gains, go for it!
                amount
                    .try_into()
                    .map_err(|_error| ProgramError::InvalidInstructionData)?,
            ),
            decimals.first(),
        )
    } else {
        return Err(ProgramError::InvalidInstructionData);
    };

    shared::burn::process_burn(accounts, amount, decimals.copied())
}
