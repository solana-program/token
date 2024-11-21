use pinocchio::{account_info::AccountInfo, msg, program_error::ProgramError, ProgramResult};
use token_interface::{error::TokenError, state::account::Account};

pub fn process_initialize_immutable_owner(accounts: &[AccountInfo]) -> ProgramResult {
    let token_account_info = accounts.first().ok_or(ProgramError::NotEnoughAccountKeys)?;

    let account = bytemuck::try_from_bytes_mut::<Account>(unsafe {
        token_account_info.borrow_mut_data_unchecked()
    })
    .map_err(|_error| ProgramError::InvalidAccountData)?;

    if account.is_initialized() {
        return Err(TokenError::AlreadyInUse.into());
    }
    msg!("Please upgrade to SPL Token 2022 for immutable owner support");
    Ok(())
}
