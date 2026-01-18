use {
    super::validate_owner,
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult},
    pinocchio_token_interface::{
        error::TokenError,
        state::{account::Account, load_mut},
    },
};

#[inline(always)]
pub fn process_revoke(accounts: &[AccountInfo]) -> ProgramResult {
    let [source_account_info, remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // SAFETY: single mutable borrow to `source_account_info` account data and
    // `load_mut` validates that the account is initialized.
    let source_account =
        unsafe { load_mut::<Account>(source_account_info.borrow_mut_data_unchecked())? };

    // Unpacking the remaining accounts to get the authority account at this point
    // to maintain the same order as SPL Token.
    let [authority_info, remaining @ ..] = remaining else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if source_account.is_frozen()? {
        return Err(TokenError::AccountFrozen.into());
    }

    // Validates the owner or delegate.

    // SAFETY: `authority_info` is not currently borrowed; in the case
    // `authority_info` is the same as `source_account_info`, then it cannot be
    // a multisig.
    unsafe {
        validate_owner(
            if source_account.delegate() == Some(authority_info.key()) {
                authority_info.key()
            } else {
                &source_account.owner
            },
            authority_info,
            remaining,
        )?
    };

    source_account.clear_delegate();
    source_account.set_delegated_amount(0);

    Ok(())
}
