use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};
use token_interface::{
    error::TokenError,
    state::{
        account::{Account, AccountState},
        mint::Mint,
    },
};

use crate::processor::validate_owner;

#[inline(always)]
pub fn process_toggle_account_state(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    freeze: bool,
) -> ProgramResult {
    let [source_account_info, mint_info, authority_info, remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let source_account = bytemuck::try_from_bytes_mut::<Account>(unsafe {
        source_account_info.borrow_mut_data_unchecked()
    })
    .map_err(|_error| ProgramError::InvalidAccountData)?;

    if freeze && source_account.is_frozen() || !freeze && !source_account.is_frozen() {
        return Err(TokenError::InvalidState.into());
    }
    if source_account.is_native.is_some() {
        return Err(TokenError::NativeNotSupported.into());
    }
    if mint_info.key() != &source_account.mint {
        return Err(TokenError::MintMismatch.into());
    }

    let mint = bytemuck::try_from_bytes::<Mint>(unsafe { mint_info.borrow_data_unchecked() })
        .map_err(|_error| ProgramError::InvalidAccountData)?;

    match mint.freeze_authority.as_ref() {
        Option::Some(authority) => validate_owner(program_id, authority, authority_info, remaining),
        Option::None => Err(TokenError::MintCannotFreeze.into()),
    }?;

    source_account.state = if freeze {
        AccountState::Frozen
    } else {
        AccountState::Initialized
    } as u8;

    Ok(())
}
