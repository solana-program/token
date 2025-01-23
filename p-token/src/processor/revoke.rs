use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};
use token_interface::{
    error::TokenError,
    state::{account::Account, load_mut},
};

use super::validate_owner;

#[inline(always)]
pub fn process_revoke(accounts: &[AccountInfo], _instruction_data: &[u8]) -> ProgramResult {
    let [source_account_info, owner_info, remaning @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // SAFETY: single mutable borrow to `source_account_info` account data and
    // `load_mut` validates that the account is initialized.
    let source_account =
        unsafe { load_mut::<Account>(source_account_info.borrow_mut_data_unchecked())? };

    if source_account.is_frozen() {
        return Err(TokenError::AccountFrozen.into());
    }

    validate_owner(&source_account.owner, owner_info, remaning)?;

    source_account.clear_delegate();
    source_account.set_delegated_amount(0);

    Ok(())
}
