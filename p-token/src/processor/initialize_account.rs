use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use std::mem::size_of;
use token_interface::{
    error::TokenError,
    native_mint::is_native_mint,
    state::{
        account::{Account, AccountState},
        mint::Mint,
        PodCOption,
    },
};

use super::check_account_owner;

pub fn process_initialize_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    owner: Option<&Pubkey>,
    rent_sysvar_account: bool,
) -> ProgramResult {
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

    let account_data = unsafe { new_account_info.borrow_mut_data_unchecked() };
    let account = bytemuck::try_from_bytes_mut::<Account>(account_data)
        .map_err(|_error| ProgramError::InvalidAccountData)?;

    if account.is_initialized() {
        return Err(TokenError::AlreadyInUse.into());
    }

    let is_native_mint = is_native_mint(mint_info.key());

    if !is_native_mint {
        check_account_owner(program_id, mint_info)?;

        let mint_data = unsafe { mint_info.borrow_data_unchecked() };
        let mint = bytemuck::try_from_bytes::<Mint>(mint_data)
            .map_err(|_error| ProgramError::InvalidAccountData)?;

        if !bool::from(mint.is_initialized) {
            return Err(TokenError::InvalidMint.into());
        }
    }

    account.mint = *mint_info.key();
    account.owner = *owner;
    account.close_authority.clear();
    account.delegate.clear();
    account.delegated_amount = 0u64.into();
    account.state = AccountState::Initialized as u8;

    if is_native_mint {
        let rent = Rent::get()?;
        let rent_exempt_reserve = rent.minimum_balance(size_of::<Account>());

        account.is_native = PodCOption::from(Some(rent_exempt_reserve.into()));
        unsafe {
            account.amount = new_account_info
                .borrow_lamports_unchecked()
                .checked_sub(rent_exempt_reserve)
                .ok_or(TokenError::Overflow)?
                .into()
        }
    } else {
        account.is_native.clear();
        account.amount = 0u64.into();
    };

    Ok(())
}
