use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};
use token_interface::{error::TokenError, state::account::Account};

use super::validate_owner;

/// Processes an [Revoke](enum.TokenInstruction.html) instruction.
pub fn process_revoke(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [source_account_info, owner_info, remaning @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let source_account = bytemuck::try_from_bytes_mut::<Account>(unsafe {
        source_account_info.borrow_mut_data_unchecked()
    })
    .map_err(|_error| ProgramError::InvalidAccountData)?;

    if source_account.is_frozen() {
        return Err(TokenError::AccountFrozen.into());
    }

    validate_owner(program_id, &source_account.owner, owner_info, remaning)?;

    source_account.delegate.clear();
    source_account.delegated_amount = 0.into();

    Ok(())
}
