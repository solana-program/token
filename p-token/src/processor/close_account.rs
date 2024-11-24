use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};
use token_interface::{error::TokenError, state::account::Account};

use super::{is_owned_by_system_program_or_incinerator, validate_owner, INCINERATOR_ID};

#[inline(always)]
pub fn process_close_account(accounts: &[AccountInfo]) -> ProgramResult {
    let [source_account_info, destination_account_info, authority_info, remaining @ ..] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if source_account_info.key() == destination_account_info.key() {
        return Err(ProgramError::InvalidAccountData);
    }

    let source_account =
        unsafe { Account::from_bytes_mut(source_account_info.borrow_mut_data_unchecked()) };

    if !source_account.is_native() && source_account.amount() != 0 {
        return Err(TokenError::NonNativeHasBalance.into());
    }

    let authority = source_account
        .close_authority()
        .unwrap_or(&source_account.owner);

    if !is_owned_by_system_program_or_incinerator(source_account_info.owner()) {
        validate_owner(authority, authority_info, remaining)?;
    } else if destination_account_info.key() != &INCINERATOR_ID {
        return Err(ProgramError::InvalidAccountData);
    }

    let destination_starting_lamports = destination_account_info.lamports();
    unsafe {
        // Moves the lamports to the destination account and closes the source account.
        *destination_account_info.borrow_mut_lamports_unchecked() = destination_starting_lamports
            .checked_add(source_account_info.lamports())
            .ok_or(TokenError::Overflow)?;

        source_account_info.close();
    }

    Ok(())
}
