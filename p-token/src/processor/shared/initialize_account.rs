use core::mem::size_of;
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use token_interface::{
    error::TokenError,
    native_mint::is_native_mint,
    state::{account::Account, account_state::AccountState, mint::Mint},
};

use crate::processor::check_account_owner;

#[inline(always)]
pub fn process_initialize_account(
    accounts: &[AccountInfo],
    owner: Option<&Pubkey>,
    rent_sysvar_account: bool,
) -> ProgramResult {
    // Accounts expected depend on whether we have the `rent_sysvar` account or not.

    let (new_account_info, mint_info, owner, remaning) = if let Some(owner) = owner {
        let [new_account_info, mint_info, remaning @ ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        (new_account_info, mint_info, owner, remaning)
    } else {
        let [new_account_info, mint_info, owner_info, remaning @ ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        (new_account_info, mint_info, owner_info.key(), remaning)
    };

    // Check rent-exempt status of the token account.

    let is_exempt = if rent_sysvar_account {
        let rent_sysvar_info = remaning.first().ok_or(ProgramError::NotEnoughAccountKeys)?;
        let rent = unsafe { Rent::from_bytes(rent_sysvar_info.borrow_data_unchecked()) };
        rent.is_exempt(new_account_info.lamports(), size_of::<Account>())
    } else {
        Rent::get()?.is_exempt(new_account_info.lamports(), size_of::<Account>())
    };

    if !is_exempt {
        return Err(TokenError::NotRentExempt.into());
    }

    // Initialize the account.

    let account = unsafe { Account::from_bytes_mut(new_account_info.borrow_mut_data_unchecked()) };

    if account.is_initialized() {
        return Err(TokenError::AlreadyInUse.into());
    }

    let is_native_mint = is_native_mint(mint_info.key());

    if !is_native_mint {
        check_account_owner(mint_info)?;

        let mint = unsafe { Mint::from_bytes(mint_info.borrow_data_unchecked()) };

        if !mint.is_initialized() {
            return Err(TokenError::InvalidMint.into());
        }
    }

    account.state = AccountState::Initialized;
    account.mint = *mint_info.key();
    account.owner = *owner;

    if is_native_mint {
        let rent = Rent::get()?;
        let rent_exempt_reserve = rent.minimum_balance(size_of::<Account>());

        account.set_native(true);
        unsafe {
            account.set_amount(
                new_account_info
                    .borrow_lamports_unchecked()
                    .checked_sub(rent_exempt_reserve)
                    .ok_or(TokenError::Overflow)?,
            );
        }
    }

    Ok(())
}
