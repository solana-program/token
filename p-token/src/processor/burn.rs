use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};
use token_interface::{
    error::TokenError,
    state::{account::Account, mint::Mint},
};

use super::{check_account_owner, is_owned_by_system_program_or_incinerator, validate_owner};

/// Processes a [Burn](enum.TokenInstruction.html) instruction.
pub fn process_burn(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    expected_decimals: Option<u8>,
) -> ProgramResult {
    let [source_account_info, mint_info, authority_info, remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Safety: There are no conflicting borrows – the source account is only borrowed once.
    let source_account = bytemuck::try_from_bytes_mut::<Account>(unsafe {
        source_account_info.borrow_mut_data_unchecked()
    })
    .map_err(|_error| ProgramError::InvalidAccountData)?;

    if source_account.is_frozen() {
        return Err(TokenError::AccountFrozen.into());
    }
    if source_account.is_native.is_some() {
        return Err(TokenError::NativeNotSupported.into());
    }

    // Ensure the source account has the sufficient amount. This is done before
    // the value is updated on the account.
    let updated_source_amount = u64::from(source_account.amount)
        .checked_sub(amount)
        .ok_or(TokenError::InsufficientFunds)?;

    // Safety: There are no conflicting borrows – the mint account is only borrowed once.
    let mint =
        bytemuck::try_from_bytes_mut::<Mint>(unsafe { mint_info.borrow_mut_data_unchecked() })
            .map_err(|_error| ProgramError::InvalidAccountData)?;

    if mint_info.key() != &source_account.mint {
        return Err(TokenError::MintMismatch.into());
    }

    if let Some(expected_decimals) = expected_decimals {
        if expected_decimals != mint.decimals {
            return Err(TokenError::MintDecimalsMismatch.into());
        }
    }

    if !is_owned_by_system_program_or_incinerator(&source_account.owner) {
        match source_account.delegate.as_ref() {
            Some(delegate) if authority_info.key() == delegate => {
                validate_owner(program_id, delegate, authority_info, remaining)?;

                let delegated_amount = u64::from(source_account.delegated_amount)
                    .checked_sub(amount)
                    .ok_or(TokenError::InsufficientFunds)?;
                source_account.delegated_amount = delegated_amount.into();

                if delegated_amount == 0 {
                    source_account.delegate.clear();
                }
            }
            _ => {
                validate_owner(program_id, &source_account.owner, authority_info, remaining)?;
            }
        }
    }

    if amount == 0 {
        check_account_owner(program_id, source_account_info)?;
        check_account_owner(program_id, mint_info)?;
    }

    source_account.amount = updated_source_amount.into();

    let mint_supply = u64::from(mint.supply)
        .checked_sub(amount)
        .ok_or(TokenError::Overflow)?;
    mint.supply = mint_supply.into();

    Ok(())
}
