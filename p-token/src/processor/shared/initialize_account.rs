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
    state::{
        account::Account, account_state::AccountState, load, load_mut_unchecked, mint::Mint,
        Initializable,
    },
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

    let new_account_info_data_len = new_account_info.data_len();

    let minimum_balance = if rent_sysvar_account {
        let rent_sysvar_info = remaning.first().ok_or(ProgramError::NotEnoughAccountKeys)?;
        let rent = unsafe { Rent::from_bytes(rent_sysvar_info.borrow_data_unchecked()) };
        rent.minimum_balance(new_account_info_data_len)
    } else {
        Rent::get()?.minimum_balance(new_account_info_data_len)
    };

    let is_native_mint = is_native_mint(mint_info.key());

    // Initialize the account.

    let account =
        unsafe { load_mut_unchecked::<Account>(new_account_info.borrow_mut_data_unchecked())? };

    if account.is_initialized() {
        return Err(TokenError::AlreadyInUse.into());
    }

    if new_account_info.lamports() < minimum_balance {
        return Err(TokenError::NotRentExempt.into());
    }

    if !is_native_mint {
        check_account_owner(mint_info)?;

        let _ = unsafe {
            load::<Mint>(mint_info.borrow_data_unchecked()).map_err(|_| TokenError::InvalidMint)?
        };
    }

    account.state = AccountState::Initialized;
    account.mint = *mint_info.key();
    account.owner = *owner;

    if is_native_mint {
        account.set_native(true);
        account.set_native_amount(minimum_balance);
        unsafe {
            account.set_amount(
                new_account_info
                    .borrow_lamports_unchecked()
                    .checked_sub(minimum_balance)
                    .ok_or(TokenError::Overflow)?,
            );
        }
    }

    Ok(())
}
