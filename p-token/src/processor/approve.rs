use {
    super::shared,
    pinocchio::{account_info::AccountInfo, ProgramResult},
    spl_token_interface::error::TokenError,
};

#[inline(always)]
pub fn process_approve(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let amount = u64::from_le_bytes(
        instruction_data
            .try_into()
            .map_err(|_error| TokenError::InvalidInstruction)?,
    );

    shared::approve::process_approve(accounts, amount, None)
}
