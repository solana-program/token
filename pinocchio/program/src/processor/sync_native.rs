use {
    super::check_account_owner,
    pinocchio::{
        account_info::AccountInfo,
        program_error::ProgramError,
        sysvars::{rent::Rent, Sysvar},
        ProgramResult,
    },
    pinocchio_token_interface::{
        error::TokenError,
        state::{account::Account, load_mut},
    },
};

#[inline(always)]
pub fn process_sync_native(accounts: &[AccountInfo]) -> ProgramResult {
    let native_account_info = accounts.first().ok_or(ProgramError::NotEnoughAccountKeys)?;

    check_account_owner(native_account_info)?;

    let rent = Rent::get()?;
    let rent_exempt_reserve = rent.minimum_balance(native_account_info.data_len());

    // SAFETY: single mutable borrow to `native_account_info` account data and
    // `load_mut` validates that the account is initialized.
    let native_account =
        unsafe { load_mut::<Account>(native_account_info.borrow_mut_data_unchecked())? };

    if native_account.is_native() {
        let new_amount = native_account_info
            .lamports()
            .checked_sub(rent_exempt_reserve)
            .ok_or(TokenError::Overflow)?;
        native_account.set_native_amount(rent_exempt_reserve);
        native_account.set_amount(new_amount);
    } else {
        return Err(TokenError::NonNativeNotSupported.into());
    }

    Ok(())
}
