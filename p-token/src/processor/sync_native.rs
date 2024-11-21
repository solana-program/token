use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};
use token_interface::{error::TokenError, state::account::Account};

use super::check_account_owner;

pub fn process_sync_native(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [native_account_info, _remaning @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    check_account_owner(program_id, native_account_info)?;

    let native_account = bytemuck::try_from_bytes_mut::<Account>(unsafe {
        native_account_info.borrow_mut_data_unchecked()
    })
    .map_err(|_error| ProgramError::InvalidAccountData)?;

    if let Option::Some(rent_exempt_reserve) = native_account.is_native.get() {
        let new_amount = native_account_info
            .lamports()
            .checked_sub(u64::from(rent_exempt_reserve))
            .ok_or(TokenError::Overflow)?;

        if new_amount < native_account.amount.into() {
            return Err(TokenError::InvalidState.into());
        }
        native_account.amount = new_amount.into();
    } else {
        return Err(TokenError::NonNativeNotSupported.into());
    }

    Ok(())
}
