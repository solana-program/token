use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};
use token_interface::{
    error::TokenError,
    state::multisig::{Multisig, MAX_SIGNERS},
};

pub mod approve;
pub mod approve_checked;
pub mod burn;
pub mod burn_checked;
pub mod close_account;
pub mod freeze_account;
pub mod initialize_account;
pub mod initialize_account2;
pub mod initialize_account3;
pub mod initialize_mint;
pub mod initialize_mint2;
pub mod initialize_multisig;
pub mod initialize_multisig2;
pub mod mint_to;
pub mod mint_to_checked;
pub mod revoke;
pub mod set_authority;
pub mod thaw_account;
pub mod transfer;
pub mod transfer_checked;
// Private processor to toggle the account state. This logic is reused by the
// freeze and thaw account instructions.
mod toggle_account_state;

/// Incinerator address.
const INCINERATOR_ID: Pubkey =
    pinocchio_pubkey::pubkey!("1nc1nerator11111111111111111111111111111111");

/// System program id.
const SYSTEM_PROGRAM_ID: Pubkey = pinocchio_pubkey::pubkey!("11111111111111111111111111111111");

#[inline(always)]
pub fn is_owned_by_system_program_or_incinerator(owner: &Pubkey) -> bool {
    SYSTEM_PROGRAM_ID == *owner || INCINERATOR_ID == *owner
}

/// Checks that the account is owned by the expected program.
#[inline(always)]
pub fn check_account_owner(program_id: &Pubkey, account_info: &AccountInfo) -> ProgramResult {
    if program_id != account_info.owner() {
        Err(ProgramError::IncorrectProgramId)
    } else {
        Ok(())
    }
}

/// Validates owner(s) are present
#[inline(always)]
pub fn validate_owner(
    program_id: &Pubkey,
    expected_owner: &Pubkey,
    owner_account_info: &AccountInfo,
    signers: &[AccountInfo],
) -> ProgramResult {
    if expected_owner != owner_account_info.key() {
        return Err(TokenError::OwnerMismatch.into());
    }

    if owner_account_info.data_len() == Multisig::LEN && program_id != owner_account_info.owner() {
        let multisig_data = owner_account_info.try_borrow_data()?;
        let multisig = bytemuck::from_bytes::<Multisig>(&multisig_data);

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
