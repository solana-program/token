use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use pinocchio::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    get_account_info,
    program_error::ProgramError,
    pubkey::{self, Pubkey},
    sysvars::{rent::Rent, Sysvar},
};

use crate::{
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
    args: Option<&InitializeAccount>,
    _rent_sysvar_account: bool,
) -> ProgramResult {
    let [new_account_info, mint_info, _remaning @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let owner = if let Some(InitializeAccount { owner }) = args {
        owner
    } else {
        get_account_info!(accounts, 2).key()
    };

    // FEBO: ~408 CU can be saved by removing the rent check (is_exempt seems to
    // be very expensive).
    //
    // The transaction will naturally fail if the account is not rent exempt with
    // a TransactionError::InsufficientFundsForRent error.
    /*
    let rent = Rent::get()?;

    if !rent.is_exempt(
        unsafe { *new_account_info.unchecked_borrow_lamports() },
        size_of::<Account>(),
    ) {
        return Err(Token::NotRentExempt);
    }
    */

    let account_data = unsafe { new_account_info.unchecked_borrow_mut_data() };
    let account = bytemuck::try_from_bytes_mut::<Account>(account_data)
        .map_err(|_error| ProgramError::InvalidAccountData)?;

    if account.is_initialized() {
        return Err(TokenError::AlreadyInUse.into());
    }

    let is_native_mint = is_native_mint(mint_info.key());

    if !is_native_mint {
        check_account_owner(program_id, mint_info)?;

        let mint_data = unsafe { mint_info.unchecked_borrow_data() };
        let mint = bytemuck::from_bytes::<Mint>(mint_data);

        let initialized: bool = mint.is_initialized.into();
        if !initialized {
            return Err(TokenError::InvalidMint.into());
        }
    }

    pubkey::copy(&mut account.mint, mint_info.key());
    pubkey::copy(&mut account.owner, owner);
    account.close_authority = PodCOption::from(None);
    account.delegate = PodCOption::from(None);
    account.delegated_amount = 0u64.to_le_bytes();
    account.state = AccountState::Initialized as u8;

    if is_native_mint {
        let rent = Rent::get()?;
        let rent_exempt_reserve = rent.minimum_balance(size_of::<Account>());

        account.is_native = PodCOption::from(Some(rent_exempt_reserve.to_le_bytes()));
        unsafe {
            account.amount = new_account_info
                .unchecked_borrow_lamports()
                .checked_sub(rent_exempt_reserve)
                .ok_or(TokenError::Overflow)?
                .to_le_bytes()
        }
    } else {
        account.is_native = PodCOption::from(None);
        account.amount = 0u64.to_le_bytes();
    };

    Ok(())
}

/// Instruction data for the `InitializeAccount` instruction.
#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
pub struct InitializeAccount {
    /// The new account's owner/multisignature.
    pub owner: Pubkey,
}
