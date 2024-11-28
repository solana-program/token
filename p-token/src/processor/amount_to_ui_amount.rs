use core::str::from_utf8_unchecked;
use pinocchio::{
    account_info::AccountInfo, program::set_return_data, program_error::ProgramError, ProgramResult,
};
use pinocchio_log::logger::{Argument, Logger};
use token_interface::{
    error::TokenError,
    state::{load, mint::Mint},
};

use super::{check_account_owner, MAX_DIGITS_U64};

/// Maximum length of the UI amount string.
///
/// The length includes the maximum number of digits in a `u64`` (20)
/// and the maximum number of punctuation characters (2).
const MAX_UI_AMOUNT_LENGTH: usize = MAX_DIGITS_U64 + 2;

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
    // SAFETY: there is a single borrow to the `Mint` account.
    let mint = unsafe {
        load::<Mint>(mint_info.borrow_data_unchecked()).map_err(|_| TokenError::InvalidMint)?
    };

    let mut logger = Logger::<MAX_UI_AMOUNT_LENGTH>::default();
    logger.append_with_args(amount, &[Argument::Precision(mint.decimals)]);
    // "Extract" the formatted string from the logger.
    //
    // SAFETY: the logger is guaranteed to be a valid UTF-8 string.
    let mut s = unsafe { from_utf8_unchecked(&logger) };

    if mint.decimals > 0 {
        let zeros_trimmed = s.trim_end_matches('0');
        s = zeros_trimmed.trim_end_matches('.');
    }

    set_return_data(s.as_bytes());

    Ok(())
}
