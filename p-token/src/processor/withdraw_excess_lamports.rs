use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use spl_token_interface::{
    error::TokenError,
    state::{account::Account, load, mint::Mint, multisig::Multisig, Transmutable},
};

use super::validate_owner;

#[inline(always)]
pub fn process_withdraw_excess_lamports(accounts: &[AccountInfo]) -> ProgramResult {
    let [source_account_info, destination_info, authority_info, remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // SAFETY: single mutable borrow to `source_account_info` account data
    let source_data = unsafe { source_account_info.borrow_data_unchecked() };

    match source_data.len() {
        Account::LEN => {
            // SAFETY: `source_data` has the same length as `Account`.
            let account = unsafe { load::<Account>(source_data)? };

            if account.is_native() {
                return Err(TokenError::NativeNotSupported.into());
            }

            validate_owner(&account.owner, authority_info, remaining)?;
        }
        Mint::LEN => {
            // SAFETY: `source_data` has the same length as `Mint`.
            let mint = unsafe { load::<Mint>(source_data)? };

            if let Some(mint_authority) = mint.mint_authority() {
                validate_owner(mint_authority, authority_info, remaining)?;
            } else {
                return Err(TokenError::AuthorityTypeNotSupported.into());
            }
        }
        Multisig::LEN => {
            validate_owner(source_account_info.key(), authority_info, remaining)?;
        }
        _ => return Err(TokenError::InvalidState.into()),
    }

    // Withdraws the excess lamports from the source account.

    let source_rent_exempt_reserve = Rent::get()?.minimum_balance(source_data.len());

    let transfer_amount = source_account_info
        .lamports()
        .checked_sub(source_rent_exempt_reserve)
        .ok_or(TokenError::NotRentExempt)?;

    let source_starting_lamports = source_account_info.lamports();
    // SAFETY: single mutable borrow to `source_account_info` lamports.
    unsafe {
        // Moves the lamports out of the source account.
        //
        // Note: The `transfer_amount` is guaranteed to be less than the source account's
        // lamports.
        *source_account_info.borrow_mut_lamports_unchecked() =
            source_starting_lamports - transfer_amount;
    }

    let destination_starting_lamports = destination_info.lamports();
    // SAFETY: single mutable borrow to `destination_info` lamports.
    unsafe {
        // Moves the lamports to the destination account.
        *destination_info.borrow_mut_lamports_unchecked() = destination_starting_lamports
            .checked_add(transfer_amount)
            .ok_or(TokenError::Overflow)?;
    }

    Ok(())
}
