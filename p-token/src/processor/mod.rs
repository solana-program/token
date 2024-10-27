use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};
use token_interface::{
    error::TokenError,
    state::multisignature::{Multisig, MAX_SIGNERS},
};

pub mod initialize_account;
pub mod initialize_mint;
pub mod mint_to;
pub mod transfer;

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
