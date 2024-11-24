use pinocchio::{
    account_info::AccountInfo, program::set_return_data, program_error::ProgramError, ProgramResult,
};
use token_interface::state::{account::Account, mint::Mint};

use super::check_account_owner;

#[inline(always)]
pub fn process_get_account_data_size(accounts: &[AccountInfo]) -> ProgramResult {
    let [mint_info, _remaning @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Make sure the mint is valid.
    check_account_owner(mint_info)?;

    let _ = unsafe { Mint::from_bytes(mint_info.borrow_data_unchecked()) };

    set_return_data(&Account::LEN.to_le_bytes());

    Ok(())
}
