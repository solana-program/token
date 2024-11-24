use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};
use token_interface::{
    error::TokenError,
    state::multisig::{Multisig, MAX_SIGNERS},
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

/// Incinerator address.
const INCINERATOR_ID: Pubkey =
    pinocchio_pubkey::pubkey!("1nc1nerator11111111111111111111111111111111");

/// System program id.
const SYSTEM_PROGRAM_ID: Pubkey = pinocchio_pubkey::pubkey!("11111111111111111111111111111111");

#[inline(always)]
fn is_owned_by_system_program_or_incinerator(owner: &Pubkey) -> bool {
    &SYSTEM_PROGRAM_ID == owner || &INCINERATOR_ID == owner
}

/// Checks that the account is owned by the expected program.
#[inline(always)]
fn check_account_owner(account_info: &AccountInfo) -> ProgramResult {
    if &crate::ID != account_info.owner() {
        Err(ProgramError::IncorrectProgramId)
    } else {
        Ok(())
    }
}

/// Validates owner(s) are present
#[inline(always)]
fn validate_owner(
    expected_owner: &Pubkey,
    owner_account_info: &AccountInfo,
    signers: &[AccountInfo],
) -> ProgramResult {
    if expected_owner != owner_account_info.key() {
        return Err(TokenError::OwnerMismatch.into());
    }

    if owner_account_info.data_len() == Multisig::LEN && &crate::ID != owner_account_info.owner() {
        let multisig = unsafe { Multisig::from_bytes(owner_account_info.borrow_data_unchecked()) };

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

/// Convert a raw amount to its UI representation using the given decimals field
/// Excess zeroes or unneeded decimal point are trimmed.
#[inline(always)]
fn amount_to_ui_amount_string_trimmed(amount: u64, decimals: u8) -> String {
    let mut s = amount_to_ui_amount_string(amount, decimals);
    if decimals > 0 {
        let zeros_trimmed = s.trim_end_matches('0');
        s = zeros_trimmed.trim_end_matches('.').to_string();
    }
    s
}

/// Convert a raw amount to its UI representation (using the decimals field
/// defined in its mint)
#[inline(always)]
fn amount_to_ui_amount_string(amount: u64, decimals: u8) -> String {
    let decimals = decimals as usize;
    if decimals > 0 {
        // Left-pad zeros to decimals + 1, so we at least have an integer zero
        let mut s = format!("{:01$}", amount, decimals + 1);
        // Add the decimal point (Sorry, "," locales!)
        s.insert(s.len() - decimals, '.');
        s
    } else {
        amount.to_string()
    }
}

/// Try to convert a UI representation of a token amount to its raw amount using
/// the given decimals field
fn try_ui_amount_into_amount(ui_amount: String, decimals: u8) -> Result<u64, ProgramError> {
    let decimals = decimals as usize;
    let mut parts = ui_amount.split('.');
    // splitting a string, even an empty one, will always yield an iterator of at
    // least length == 1
    let mut amount_str = parts.next().unwrap().to_string();
    let after_decimal = parts.next().unwrap_or("");
    let after_decimal = after_decimal.trim_end_matches('0');
    if (amount_str.is_empty() && after_decimal.is_empty())
        || parts.next().is_some()
        || after_decimal.len() > decimals
    {
        return Err(ProgramError::InvalidArgument);
    }

    amount_str.push_str(after_decimal);
    for _ in 0..decimals.saturating_sub(after_decimal.len()) {
        amount_str.push('0');
    }
    amount_str
        .parse::<u64>()
        .map_err(|_| ProgramError::InvalidArgument)
}
