use core::{
    cmp::max,
    mem::MaybeUninit,
    slice::{from_raw_parts, from_raw_parts_mut},
    str::from_utf8_unchecked,
};
use pinocchio::{
    account_info::AccountInfo, memory::sol_memcpy, program_error::ProgramError, pubkey::Pubkey,
    ProgramResult,
};
use token_interface::{
    error::TokenError,
    program::ID as TOKEN_PROGRAM_ID,
    state::{
        load,
        multisig::{Multisig, MAX_SIGNERS},
        RawType,
    },
};

pub mod amount_to_ui_amount;
pub mod approve;
pub mod approve_checked;
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
// Shared processors.
pub mod shared;

pub use amount_to_ui_amount::process_amount_to_ui_amount;
pub use approve::process_approve;
pub use approve_checked::process_approve_checked;
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

/// An uninitialized byte.
const UNINIT_BYTE: MaybeUninit<u8> = MaybeUninit::uninit();

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
/// a multisig account.
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
        // account data and the `load` validates that the account is initialized.
        let multisig = unsafe { load::<Multisig>(owner_account_info.borrow_data_unchecked())? };

        let mut num_signers = 0;
        let mut matched = [false; MAX_SIGNERS];

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

    let mut length = amount_str.len();
    let expected_after_decimal_length = max(after_decimal.len(), decimals);

    if (amount_str.is_empty() && after_decimal.is_empty())
        || parts.next().is_some()
        || after_decimal.len() > decimals
        || (length + expected_after_decimal_length) > MAX_FORMATTED_DIGITS
    {
        return Err(ProgramError::InvalidArgument);
    }

    let mut digits = [UNINIT_BYTE; MAX_FORMATTED_DIGITS];
    // SAFETY: `digits` is an array of `MaybeUninit<u8>`, which has the same
    // memory layout as `u8`.
    let slice: &mut [u8] =
        unsafe { from_raw_parts_mut(digits.as_mut_ptr() as *mut _, MAX_FORMATTED_DIGITS) };

    // SAFETY: the total length of `amount_str` and `after_decimal` is less than
    // `MAX_DIGITS_U64`.
    unsafe {
        sol_memcpy(slice, amount_str.as_bytes(), length);

        sol_memcpy(
            &mut slice[length..],
            after_decimal.as_bytes(),
            after_decimal.len(),
        );
    }

    length += after_decimal.len();
    let remaining = decimals.saturating_sub(after_decimal.len());

    // SAFETY: `digits` is an array of `MaybeUninit<u8>`, which has the same memory
    // layout as `u8`.
    let ptr = unsafe { digits.as_mut_ptr().add(length) };

    for offset in 0..remaining {
        // SAFETY: `ptr` is within the bounds of `digits`.
        unsafe {
            (ptr.add(offset) as *mut u8).write(b'0');
        }
    }

    length += remaining;

    // SAFETY: `digits` only contains valid UTF-8 bytes.
    unsafe {
        from_utf8_unchecked(from_raw_parts(digits.as_ptr() as _, length))
            .parse::<u64>()
            .map_err(|_| ProgramError::InvalidArgument)
    }
}
