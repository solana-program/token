use pinocchio::{
    account_info::AccountInfo, program::set_return_data, program_error::ProgramError, ProgramResult,
};
use token_interface::state::mint::Mint;

use super::{amount_to_ui_amount_string_trimmed, check_account_owner};

#[inline(always)]
pub fn process_amount_to_ui_amount(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let amount = u64::from_le_bytes(
        instruction_data
            .try_into()
            .map_err(|_error| ProgramError::InvalidInstructionData)?,
    );

    let mint_info = accounts.first().ok_or(ProgramError::NotEnoughAccountKeys)?;
    check_account_owner(mint_info)?;

    let mint = unsafe { Mint::from_bytes(mint_info.borrow_data_unchecked()) };

    let ui_amount = amount_to_ui_amount_string_trimmed(amount, mint.decimals);
    set_return_data(&ui_amount.into_bytes());

    Ok(())
}
