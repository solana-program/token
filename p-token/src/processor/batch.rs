use {
    crate::{entrypoint::inner_process_instruction, processor::check_account_owner},
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult},
    pinocchio_token_interface::error::TokenError,
};

/// The size of the batch instruction header.
///
/// The header of each instruction consists of two `u8` values:
///  * number of the accounts
///  * length of the instruction data
const IX_HEADER_SIZE: usize = 2;

#[allow(clippy::arithmetic_side_effects)]
pub fn process_batch(mut accounts: &[AccountInfo], mut instruction_data: &[u8]) -> ProgramResult {
    loop {
        // Validates the instruction data and accounts offset.

        if instruction_data.len() < IX_HEADER_SIZE {
            // The instruction data must have at least two bytes.
            return Err(TokenError::InvalidInstruction.into());
        }

        // SAFETY: The instruction data is guaranteed to have at least two bytes
        // (header) + one byte (discriminator) and the values are within the bounds
        // of an `usize`.
        let expected_accounts = unsafe { *instruction_data.get_unchecked(0) as usize };
        let data_offset = IX_HEADER_SIZE + unsafe { *instruction_data.get_unchecked(1) as usize };

        if instruction_data.len() < data_offset || data_offset == IX_HEADER_SIZE {
            return Err(TokenError::InvalidInstruction.into());
        }

        if accounts.len() < expected_accounts {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        // Process the instruction.

        // SAFETY: The instruction data and accounts lengths are already validated so
        // all slices are guaranteed to be valid.
        let (ix_accounts, ix_data) = unsafe {
            (
                accounts.get_unchecked(..expected_accounts),
                instruction_data.get_unchecked(IX_HEADER_SIZE..data_offset),
            )
        };

        // `Transfer` and `TransferChecked` instructions require specific account
        // ownership checks when executed in a batch since account ownership is
        // checked by the runtime at the end of the batch processing only.
        match ix_data.first() {
            // 3 - Transfer
            Some(3) => {
                let [source_account_info, destination_account_info, _remaining @ ..] = ix_accounts
                else {
                    return Err(ProgramError::NotEnoughAccountKeys);
                };

                check_account_owner(source_account_info)?;
                check_account_owner(destination_account_info)?;
            }
            // 12 - TransferChecked
            Some(12) => {
                let [source_account_info, _, destination_account_info, _remaining @ ..] =
                    ix_accounts
                else {
                    return Err(ProgramError::NotEnoughAccountKeys);
                };

                check_account_owner(source_account_info)?;
                check_account_owner(destination_account_info)?;
            }
            _ => (),
        }

        inner_process_instruction(ix_accounts, ix_data)?;

        if data_offset == instruction_data.len() {
            // The batch is complete.
            break;
        }

        accounts = &accounts[expected_accounts..];
        instruction_data = &instruction_data[data_offset..];
    }

    Ok(())
}
