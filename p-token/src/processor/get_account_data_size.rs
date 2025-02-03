use pinocchio::{
    account_info::AccountInfo, program::set_return_data, program_error::ProgramError, ProgramResult,
};
use token_interface::{
    error::TokenError,
    state::{account::Account, load, mint::Mint, RawType},
};

use super::check_account_owner;

#[inline(always)]
pub fn process_get_account_data_size(accounts: &[AccountInfo]) -> ProgramResult {
    let [mint_info, _remaning @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Make sure the mint is valid.
    check_account_owner(mint_info)?;

    // SAFETY: single immutable borrow to `mint_info` account data and
    // `load` validates that the mint is initialized.
    let _ = unsafe {
        load::<Mint>(mint_info.borrow_data_unchecked()).map_err(|_| TokenError::InvalidMint)?
    };

    set_return_data(&Account::LEN.to_le_bytes());

    Ok(())
}
