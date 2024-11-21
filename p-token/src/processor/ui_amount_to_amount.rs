use pinocchio::{
    account_info::AccountInfo, program::set_return_data, program_error::ProgramError,
    pubkey::Pubkey, ProgramResult,
};
use token_interface::{error::TokenError, state::mint::Mint};

use super::{check_account_owner, try_ui_amount_into_amount};

pub fn process_ui_amount_to_amount(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    ui_amount: &str,
) -> ProgramResult {
    let mint_info = accounts.first().ok_or(ProgramError::NotEnoughAccountKeys)?;
    check_account_owner(program_id, mint_info)?;

    let mint =
        bytemuck::try_from_bytes_mut::<Mint>(unsafe { mint_info.borrow_mut_data_unchecked() })
            .map_err(|_error| TokenError::InvalidMint)?;

    let amount = try_ui_amount_into_amount(ui_amount.to_string(), mint.decimals)?;
    set_return_data(&amount.to_le_bytes());

    Ok(())
}
