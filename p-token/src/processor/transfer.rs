use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};
use token_interface::{
    error::TokenError,
    native_mint::is_native_mint,
    state::{account::Account, mint::Mint, PodCOption},
};

use super::{check_account_owner, validate_owner};

pub fn process_transfer(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    expected_decimals: Option<u8>,
) -> ProgramResult {
    // Accounts expected depends on whether we have the mint `decimals` or not; when we have the
    // mint `decimals`, we expect the mint account to be present.

    let (
        source_account_info,
        expected_mint_info,
        destination_account_info,
        authority_info,
        remaning,
    ) = if let Some(decimals) = expected_decimals {
        let [source_account_info, mint_info, destination_account_info, authority_info, remaning @ ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        (
            source_account_info,
            Some((mint_info, decimals)),
            destination_account_info,
            authority_info,
            remaning,
        )
    } else {
        let [source_account_info, destination_account_info, authority_info, remaning @ ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        (
            source_account_info,
            None,
            destination_account_info,
            authority_info,
            remaning,
        )
    };

    // Validates source and destination accounts.

    let source_account = bytemuck::try_from_bytes_mut::<Account>(unsafe {
        source_account_info.borrow_mut_data_unchecked()
    })
    .map_err(|_error| ProgramError::InvalidAccountData)?;

    let destination_account = bytemuck::try_from_bytes_mut::<Account>(unsafe {
        destination_account_info.borrow_mut_data_unchecked()
    })
    .map_err(|_error| ProgramError::InvalidAccountData)?;

    if source_account.is_frozen() || destination_account.is_frozen() {
        return Err(TokenError::AccountFrozen.into());
    }

    // FEBO: Implicitly validates that the account has enough tokens by calculating the
    // remaining amount. The amount is only updated on the account if the transfer
    // is successful.
    let remaining_amount = u64::from(source_account.amount)
        .checked_sub(amount)
        .ok_or(TokenError::InsufficientFunds)?;

    if source_account.mint != destination_account.mint {
        return Err(TokenError::MintMismatch.into());
    }

    // Validates the mint information.

    if let Some((mint_info, decimals)) = expected_mint_info {
        if mint_info.key() != &source_account.mint {
            return Err(TokenError::MintMismatch.into());
        }

        let mint =
            bytemuck::try_from_bytes_mut::<Mint>(unsafe { mint_info.borrow_mut_data_unchecked() })
                .map_err(|_error| ProgramError::InvalidAccountData)?;

        if decimals != mint.decimals {
            return Err(TokenError::MintDecimalsMismatch.into());
        }
    }

    let self_transfer = source_account_info.key() == destination_account_info.key();

    // Validates the authority (delegate or owner).

    if source_account.delegate.as_ref() == Some(authority_info.key()) {
        validate_owner(program_id, authority_info.key(), authority_info, remaning)?;

        let delegated_amount = u64::from(source_account.delegated_amount)
            .checked_sub(amount)
            .ok_or(TokenError::InsufficientFunds)?;

        if !self_transfer {
            source_account.delegated_amount = delegated_amount.into();

            if delegated_amount == 0 {
                source_account.delegate = PodCOption::from(None);
            }
        }
    } else {
        validate_owner(program_id, &source_account.owner, authority_info, remaning)?;
    }

    if self_transfer || amount == 0 {
        check_account_owner(program_id, source_account_info)?;
        check_account_owner(program_id, destination_account_info)?;

        // No need to move tokens around.
        return Ok(());
    }

    // FEBO: This was moved to the if statement above since we can skip the amount
    // manipulation if it is a self-transfer or the amount is zero.
    //
    // This check MUST occur just before the amounts are manipulated
    // to ensure self-transfers are fully validated
    /*
    if self_transfer {
        return Ok(());
    }
    */

    // Moves the tokens.

    source_account.amount = remaining_amount.into();

    let destination_amount = u64::from(destination_account.amount)
        .checked_add(amount)
        .ok_or(TokenError::Overflow)?;
    destination_account.amount = destination_amount.into();

    if is_native_mint(&source_account.mint) {
        let mut source_lamports = source_account_info.try_borrow_mut_lamports()?;
        *source_lamports = source_lamports
            .checked_sub(amount)
            .ok_or(TokenError::Overflow)?;

        let mut destination_lamports = destination_account_info.try_borrow_mut_lamports()?;
        *destination_lamports = destination_lamports
            .checked_add(amount)
            .ok_or(TokenError::Overflow)?;
    }

    Ok(())
}
