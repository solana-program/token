use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};
use token_interface::{
    error::TokenError,
    state::{account::Account, mint::Mint, PodCOption},
};

use crate::processor::validate_owner;

#[inline(always)]
pub fn process_approve(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    expected_decimals: Option<u8>,
) -> ProgramResult {
    let (source_account_info, expected_mint_info, delegate_info, owner_info, remaining) =
        if let Some(expected_decimals) = expected_decimals {
            let [source_account_info, expected_mint_info, delegate_info, owner_info, remaning @ ..] =
                accounts
            else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };

            (
                source_account_info,
                Some((expected_mint_info, expected_decimals)),
                delegate_info,
                owner_info,
                remaning,
            )
        } else {
            let [source_account_info, delegate_info, owner_info, remaning @ ..] = accounts else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };
            (
                source_account_info,
                None,
                delegate_info,
                owner_info,
                remaning,
            )
        };

    let source_account = bytemuck::try_from_bytes_mut::<Account>(unsafe {
        source_account_info.borrow_mut_data_unchecked()
    })
    .map_err(|_error| ProgramError::InvalidAccountData)?;

    if source_account.is_frozen() {
        return Err(TokenError::AccountFrozen.into());
    }

    if let Some((mint_info, expected_decimals)) = expected_mint_info {
        if mint_info.key() != &source_account.mint {
            return Err(TokenError::MintMismatch.into());
        }

        let mint = bytemuck::try_from_bytes::<Mint>(unsafe { mint_info.borrow_data_unchecked() })
            .map_err(|_error| ProgramError::InvalidAccountData)?;

        if expected_decimals != mint.decimals {
            return Err(TokenError::MintDecimalsMismatch.into());
        }
    }

    validate_owner(program_id, &source_account.owner, owner_info, remaining)?;

    source_account.delegate = PodCOption::some(*delegate_info.key());
    source_account.delegated_amount = amount.into();

    Ok(())
}
