use core::{slice::from_raw_parts, str::from_utf8_unchecked};
use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, syscalls::sol_memcpy_,
    ProgramResult,
};
use spl_token_interface::{
    error::TokenError,
    program::ID as TOKEN_PROGRAM_ID,
    state::{
        load,
        multisig::{Multisig, MAX_SIGNERS},
        Transmutable,
    },
};

pub mod amount_to_ui_amount;
pub mod approve;
pub mod approve_checked;
pub mod batch;
pub mod burn;
pub mod burn_checked;
pub mod close_account;
pub mod freeze_account;
pub mod get_account_data_size;
pub mod initialize_account;
pub mod initialize_account2;
pub mod initialize_account3;
pub mod initialize_immutable_owner;
pub mod initialize_mint;
pub mod initialize_mint2;
pub mod initialize_multisig;
pub mod initialize_multisig2;
pub mod mint_to;
pub mod mint_to_checked;
pub mod revoke;
pub mod set_authority;
pub mod sync_native;
pub mod thaw_account;
pub mod transfer;
pub mod transfer_checked;
pub mod ui_amount_to_amount;
pub mod withdraw_excess_lamports;
// Shared processors.
pub mod shared;

pub use amount_to_ui_amount::process_amount_to_ui_amount;
pub use approve::process_approve;
pub use approve_checked::process_approve_checked;
pub use batch::process_batch;
pub use burn::process_burn;
pub use burn_checked::process_burn_checked;
pub use close_account::process_close_account;
pub use freeze_account::process_freeze_account;
pub use get_account_data_size::process_get_account_data_size;
pub use initialize_account::process_initialize_account;
pub use initialize_account2::process_initialize_account2;
pub use initialize_account3::process_initialize_account3;
pub use initialize_immutable_owner::process_initialize_immutable_owner;
pub use initialize_mint::process_initialize_mint;
pub use initialize_mint2::process_initialize_mint2;
pub use initialize_multisig::process_initialize_multisig;
pub use initialize_multisig2::process_initialize_multisig2;
pub use mint_to::process_mint_to;
pub use mint_to_checked::process_mint_to_checked;
pub use revoke::process_revoke;
pub use set_authority::process_set_authority;
pub use sync_native::process_sync_native;
pub use thaw_account::process_thaw_account;
pub use transfer::process_transfer;
pub use transfer_checked::process_transfer_checked;
pub use ui_amount_to_amount::process_ui_amount_to_amount;
pub use withdraw_excess_lamports::process_withdraw_excess_lamports;

/// Maximum number of digits in a formatted `u64`.
///
/// The maximum number of digits is equal to the maximum number
/// of decimals (`u8::MAX`) plus the length of the decimal point
/// and the leading zero.
const MAX_FORMATTED_DIGITS: usize = u8::MAX as usize + 2;

/// Checks that the account is owned by the expected program.
#[inline(always)]
fn check_account_owner(account_info: &AccountInfo) -> ProgramResult {
    if &TOKEN_PROGRAM_ID != account_info.owner() {
        Err(ProgramError::IncorrectProgramId)
    } else {
        Ok(())
    }
}

/// Validates owner(s) are present.
///
/// Note that `owner_account_info` will be immutable borrowed when it represents
/// a multisig account, therefore it should not have any mutable borrows when
/// calling this function.
#[inline(always)]
fn validate_owner(
    expected_owner: &Pubkey,
    owner_account_info: &AccountInfo,
    signers: &[AccountInfo],
) -> ProgramResult {
    if expected_owner != owner_account_info.key() {
        return Err(TokenError::OwnerMismatch.into());
    }

    if owner_account_info.data_len() == Multisig::LEN
        && owner_account_info.owner() == &TOKEN_PROGRAM_ID
    {
        // SAFETY: the caller guarantees that there are no mutable borrows of `owner_account_info`
        // account data and the `load` validates that the account is initialized; additionally,
        // `Multisig` accounts are only ever loaded in this function, which means that previous
        // loads will have already failed by the time we get here.
        let multisig = unsafe { load::<Multisig>(owner_account_info.borrow_data_unchecked())? };

        let mut num_signers = 0;
        let mut matched = [false; MAX_SIGNERS as usize];

        for signer in signers.iter() {
            for (position, key) in multisig.signers[0..multisig.n as usize].iter().enumerate() {
                if key == signer.key() && !matched[position] {
                    if !signer.is_signer() {
                        return Err(ProgramError::MissingRequiredSignature);
                    }
                    matched[position] = true;
                    num_signers += 1;
                }
            }
        }
        if num_signers < multisig.m {
            return Err(ProgramError::MissingRequiredSignature);
        }
    } else if !owner_account_info.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    Ok(())
}

/// Try to convert a UI representation of a token amount to its raw amount using
/// the given decimals field
fn try_ui_amount_into_amount(ui_amount: &str, decimals: u8) -> Result<u64, ProgramError> {
    let decimals = decimals as usize;
    let mut parts = ui_amount.split('.');

    // Splitting a string, even an empty one, will always yield an iterator of at
    // least length == 1.
    let amount_str = parts.next().unwrap();
    let after_decimal = parts.next().unwrap_or("");
    // Clean up trailing zeros.
    let after_decimal = after_decimal.trim_end_matches('0');

    // Validates the input.

    let length = amount_str.len();

    if (amount_str.is_empty() && after_decimal.is_empty())
        || parts.next().is_some()
        || after_decimal.len() > decimals
        || (length + decimals) > MAX_FORMATTED_DIGITS
    {
        return Err(ProgramError::InvalidArgument);
    }

    let mut digits = [b'0'; MAX_FORMATTED_DIGITS];

    // SAFETY: the total length of `amount_str` and `after_decimal` is less than
    // `MAX_FORMATTED_DIGITS`.
    unsafe {
        sol_memcpy_(digits.as_mut_ptr(), amount_str.as_ptr(), length as u64);

        sol_memcpy_(
            digits.as_mut_ptr().add(length),
            after_decimal.as_ptr(),
            after_decimal.len() as u64,
        );
    }

    let length = amount_str.len() + decimals;

    // SAFETY: `digits` only contains valid UTF-8 bytes.
    unsafe {
        from_utf8_unchecked(from_raw_parts(digits.as_ptr(), length))
            .parse::<u64>()
            .map_err(|_| ProgramError::InvalidArgument)
    }
}
