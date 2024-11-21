use pinocchio::{
    account_info::AccountInfo, program::set_return_data, program_error::ProgramError,
    pubkey::Pubkey, ProgramResult,
};
use token_interface::state::{account::Account, mint::Mint};

use super::check_account_owner;

#[inline(never)]
pub fn process_get_account_data_size(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [mint_info, _remaning @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    check_account_owner(program_id, mint_info)?;

    let _ = bytemuck::try_from_bytes::<Mint>(unsafe { mint_info.borrow_data_unchecked() })
        .map_err(|_error| ProgramError::InvalidAccountData)?;

    set_return_data(&Account::LEN.to_le_bytes());

    Ok(())
}
