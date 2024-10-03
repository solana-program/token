use pinocchio::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    error::TokenError,
    native_mint::is_native_mint,
    state::{account::Account, mint::Mint},
};

use super::{check_account_owner, validate_owner};

pub fn process_mint_to(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    expected_decimals: Option<u8>,
) -> ProgramResult {
    let [mint_info, destination_account_info, owner_info, remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // destination account

    let account_data = unsafe { destination_account_info.unchecked_borrow_mut_data() };
    let destination_account = bytemuck::from_bytes_mut::<Account>(account_data);

    if destination_account.is_frozen() {
        return Err(TokenError::AccountFrozen.into());
    }

    if is_native_mint(mint_info.key()) {
        return Err(TokenError::NativeNotSupported.into());
    }

    if mint_info.key() != &destination_account.mint {
        return Err(TokenError::MintMismatch.into());
    }

    let mint_data = unsafe { mint_info.unchecked_borrow_mut_data() };
    let mint = bytemuck::from_bytes_mut::<Mint>(mint_data);

    if let Some(expected_decimals) = expected_decimals {
        if expected_decimals != mint.decimals {
            return Err(TokenError::MintDecimalsMismatch.into());
        }
    }

    match mint.mint_authority.get() {
        Some(mint_authority) => validate_owner(program_id, &mint_authority, owner_info, remaining)?,
        None => return Err(TokenError::FixedSupply.into()),
    }

    if amount == 0 {
        check_account_owner(program_id, mint_info)?;
        check_account_owner(program_id, destination_account_info)?;
    }

    let destination_amount = u64::from_le_bytes(destination_account.amount)
        .checked_add(amount)
        .ok_or(ProgramError::InvalidAccountData)?;
    destination_account.amount = destination_amount.to_le_bytes();

    let mint_supply = u64::from_le_bytes(mint.supply)
        .checked_add(amount)
        .ok_or(ProgramError::InvalidAccountData)?;
    mint.supply = mint_supply.to_le_bytes();

    Ok(())
}
