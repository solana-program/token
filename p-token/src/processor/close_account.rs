use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};
use token_interface::{error::TokenError, state::account::Account};

use super::{is_owned_by_system_program_or_incinerator, validate_owner, INCINERATOR_ID};

#[inline(never)]
pub fn process_close_account(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [source_account_info, destination_account_info, authority_info, remaining @ ..] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if source_account_info.key() == destination_account_info.key() {
        return Err(ProgramError::InvalidAccountData);
    }

    let source_account =
        bytemuck::try_from_bytes::<Account>(unsafe { source_account_info.borrow_data_unchecked() })
            .map_err(|_error| ProgramError::InvalidAccountData)?;

    if source_account.is_native.is_none() && source_account.amount() != 0 {
        return Err(TokenError::NonNativeHasBalance.into());
    }

    let authority = source_account
        .close_authority
        .get()
        .unwrap_or(source_account.owner);

    if !is_owned_by_system_program_or_incinerator(source_account_info.owner()) {
        validate_owner(program_id, &authority, authority_info, remaining)?;
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
