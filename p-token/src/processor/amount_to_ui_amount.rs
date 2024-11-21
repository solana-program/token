use pinocchio::{
    account_info::AccountInfo, program::set_return_data, program_error::ProgramError,
    pubkey::Pubkey, ProgramResult,
};
use token_interface::{error::TokenError, state::mint::Mint};

use super::{amount_to_ui_amount_string_trimmed, check_account_owner};

#[inline(never)]
pub fn process_amount_to_ui_amount(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let mint_info = accounts.first().ok_or(ProgramError::NotEnoughAccountKeys)?;
    check_account_owner(program_id, mint_info)?;

    let mint =
        bytemuck::try_from_bytes_mut::<Mint>(unsafe { mint_info.borrow_mut_data_unchecked() })
            .map_err(|_error| TokenError::InvalidMint)?;

    let ui_amount = amount_to_ui_amount_string_trimmed(amount, mint.decimals);
    set_return_data(&ui_amount.into_bytes());

    Ok(())
}
