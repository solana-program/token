use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};
use token_interface::{
    error::TokenError,
    state::{account::Account, load, load_mut, load_mut_unchecked, mint::Mint},
};

use crate::processor::{check_account_owner, validate_owner};

#[inline(always)]
pub fn process_transfer(
    accounts: &[AccountInfo],
    amount: u64,
    expected_decimals: Option<u8>,
) -> ProgramResult {
    // Accounts expected depend on whether we have the mint `decimals` or not; when we have the
    // mint `decimals`, we expect the mint account to be present.

    let (
        source_account_info,
        expected_mint_info,
        destination_account_info,
        authority_info,
        remaning,
    ) = if let Some(decimals) = expected_decimals {
        let [source_account_info, mint_info, destination_account_info, authority_info, remaning @ ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        (
            source_account_info,
            Some((mint_info, decimals)),
            destination_account_info,
            authority_info,
            remaning,
        )
    } else {
        let [source_account_info, destination_account_info, authority_info, remaning @ ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        (
            source_account_info,
            None,
            destination_account_info,
            authority_info,
            remaning,
        )
    };

    // Validates source and destination accounts.

    // SAFETY: single mutable borrow to `source_account_info` account data and
    // `load_mut` validates that the account is initialized.
    let source_account =
        unsafe { load_mut::<Account>(source_account_info.borrow_mut_data_unchecked())? };

    // Comparing whether the AccountInfo's "point" to the same account or
    // not - this is a faster comparison since it just checks the internal
    // raw pointer.
    let self_transfer = source_account_info == destination_account_info;

    // Implicitly validates that the account has enough tokens by calculating the
    // remaining amount - the amount is only updated on the account if the transfer
    // is successful.
    let remaining_amount = if self_transfer {
        if source_account.is_frozen() {
            return Err(TokenError::AccountFrozen.into());
        }

        source_account
            .amount()
            .checked_sub(amount)
            .ok_or(TokenError::InsufficientFunds)?
    } else {
        // SAFETY: scoped immutable borrow to `destination_account_info` account data and
        // `load` validates that the account is initialized.
        let destination_account =
            unsafe { load::<Account>(destination_account_info.borrow_data_unchecked())? };

        if source_account.is_frozen() || destination_account.is_frozen() {
            return Err(TokenError::AccountFrozen.into());
        }

        let remaining_amount = source_account
            .amount()
            .checked_sub(amount)
            .ok_or(TokenError::InsufficientFunds)?;

        if source_account.mint != destination_account.mint {
            return Err(TokenError::MintMismatch.into());
        }

        remaining_amount
    };

    // Validates the mint information.

    if let Some((mint_info, decimals)) = expected_mint_info {
        if mint_info.key() != &source_account.mint {
            return Err(TokenError::MintMismatch.into());
        }

        // SAFETY: single immutable borrow of `mint_info` account data and
        // `load` validates that the mint is initialized.
        let mint = unsafe { load::<Mint>(mint_info.borrow_data_unchecked())? };

        if decimals != mint.decimals {
            return Err(TokenError::MintDecimalsMismatch.into());
        }
    }

    // Validates the authority (delegate or owner).

    if source_account.delegate() == Some(authority_info.key()) {
        validate_owner(authority_info.key(), authority_info, remaning)?;

        let delegated_amount = source_account
            .delegated_amount()
            .checked_sub(amount)
            .ok_or(TokenError::InsufficientFunds)?;

        if !self_transfer {
            source_account.set_delegated_amount(delegated_amount);

            if delegated_amount == 0 {
                source_account.clear_delegate();
            }
        }
    } else {
        validate_owner(&source_account.owner, authority_info, remaning)?;
    }

    if self_transfer || amount == 0 {
        // Validates the token accounts owner since we are not writing
        // to these account.
        check_account_owner(source_account_info)?;
        check_account_owner(destination_account_info)?;
    } else {
        // Moves the tokens.

        source_account.set_amount(remaining_amount);

        // SAFETY: single mutable borrow to `destination_account_info` account data; the account
        // is guaranteed to be initialized and different than `source_account_info`.
        let destination_account = unsafe {
            load_mut_unchecked::<Account>(destination_account_info.borrow_mut_data_unchecked())?
        };
        let destination_amount = destination_account
            .amount()
            .checked_add(amount)
            .ok_or(TokenError::Overflow)?;
        destination_account.set_amount(destination_amount);

        if source_account.is_native() {
            // SAFETY: single mutable borrow to `source_account_info` lamports.
            let source_lamports = unsafe { source_account_info.borrow_mut_lamports_unchecked() };
            *source_lamports = source_lamports
                .checked_sub(amount)
                .ok_or(TokenError::Overflow)?;

            // SAFETY: single mutable borrow to `destination_account_info` lamports.
            let destination_lamports =
                unsafe { destination_account_info.borrow_mut_lamports_unchecked() };
            *destination_lamports = destination_lamports
                .checked_add(amount)
                .ok_or(TokenError::Overflow)?;
        }
    }

    Ok(())
}
