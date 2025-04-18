use {
    super::shared,
    pinocchio::{account_info::AccountInfo, pubkey::Pubkey, ProgramResult},
    spl_token_interface::error::TokenError,
};

#[inline(always)]
pub fn process_initialize_account2(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let owner: &Pubkey = instruction_data
        .try_into()
        .map_err(|_error| TokenError::InvalidInstruction)?;

    shared::initialize_account::process_initialize_account(accounts, Some(owner), true)
}
