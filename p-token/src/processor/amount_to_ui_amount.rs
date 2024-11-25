use core::str::from_utf8_unchecked;
use pinocchio::{
    account_info::AccountInfo, program::set_return_data, program_error::ProgramError, ProgramResult,
};
use pinocchio_log::logger::{Argument, Logger};
use token_interface::state::mint::Mint;

use super::{check_account_owner, MAX_DIGITS_U64};

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

    let mut logger = Logger::<MAX_DIGITS_U64>::default();
    logger.append_with_args(amount, &[Argument::Precision(mint.decimals)]);

    let mut s = unsafe { from_utf8_unchecked(&logger) };

    if mint.decimals > 0 {
        let zeros_trimmed = s.trim_end_matches('0');
        s = zeros_trimmed.trim_end_matches('.');
    }

    set_return_data(s.as_bytes());

    Ok(())
}
