use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

use crate::entrypoint::inner_process_instruction;

/// The size of the batch instruction header.
///
/// The header of each instruction consists of two `u8` values:
///  * number of the accounts
///  * length of the instruction data
const IX_HEADER_SIZE: usize = 2;

pub fn process_batch(mut accounts: &[AccountInfo], mut instruction_data: &[u8]) -> ProgramResult {
    loop {
        // Validates the instruction data and accounts offset.

        if instruction_data.len() < IX_HEADER_SIZE {
            // The instruction data must have at least two bytes.
            return Err(ProgramError::InvalidInstructionData);
        }

        // SAFETY: The instruction data is guaranteed to have at least two bytes (header)
        // + one byte (discriminator).
        let expected_accounts = unsafe { *instruction_data.get_unchecked(0) as usize };
        let data_offset = IX_HEADER_SIZE + unsafe { *instruction_data.get_unchecked(1) as usize };

        if instruction_data.len() < data_offset || data_offset == 0 {
            return Err(ProgramError::InvalidInstructionData);
        }

        if accounts.len() < expected_accounts {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        // Process the instruction.

        // SAFETY: The instruction data and accounts lengths are already validated so all
        // the slices are guaranteed to be valid.
        unsafe {
            inner_process_instruction(
                accounts.get_unchecked(..expected_accounts),
                instruction_data.get_unchecked(IX_HEADER_SIZE..data_offset),
                //*instruction_data.get_unchecked(IX_HEADER_SIZE),
            )?;
        }

        if data_offset == instruction_data.len() {
            // The batch is complete.
            break;
        }

        // SAFETY: Both `accounts` and `instruction_data` will have at least the
        // expected number of accounts and the data offset, respectively.
        //accounts = unsafe { accounts.get_unchecked(expected_accounts..) };
        //instruction_data = unsafe { instruction_data.get_unchecked(data_offset..) };
        accounts = &accounts[expected_accounts..];
        instruction_data = &instruction_data[data_offset..];
    }

    Ok(())
}
