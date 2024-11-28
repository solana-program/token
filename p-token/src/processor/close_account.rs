use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};
use token_interface::{
    error::TokenError,
    state::{account::Account, load_mut},
};

use super::validate_owner;

/// Incinerator address.
const INCINERATOR_ID: Pubkey =
    pinocchio_pubkey::pubkey!("1nc1nerator11111111111111111111111111111111");

#[inline(always)]
pub fn process_close_account(accounts: &[AccountInfo]) -> ProgramResult {
    let [source_account_info, destination_account_info, authority_info, remaining @ ..] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Comparing whether the AccountInfo's "point" to the same account or
    // not - this is a faster comparison since it just checks the internal
    // raw pointer.
    if source_account_info == destination_account_info {
        return Err(ProgramError::InvalidAccountData);
    }

    let source_account =
        unsafe { load_mut::<Account>(source_account_info.borrow_mut_data_unchecked())? };

    if !source_account.is_native() && source_account.amount() != 0 {
        return Err(TokenError::NonNativeHasBalance.into());
    }

    let authority = source_account
        .close_authority()
        .unwrap_or(&source_account.owner);

    if !source_account.is_owned_by_system_program_or_incinerator() {
        validate_owner(authority, authority_info, remaining)?;
    } else if destination_account_info.key() != &INCINERATOR_ID {
        return Err(ProgramError::InvalidAccountData);
    }

    let destination_starting_lamports = destination_account_info.lamports();
    unsafe {
        // Moves the lamports to the destination account.
        *destination_account_info.borrow_mut_lamports_unchecked() = destination_starting_lamports
            .checked_add(source_account_info.lamports())
            .ok_or(TokenError::Overflow)?;
        // Closes the source account.
        source_account_info.close_unchecked();
    }

    Ok(())
}
