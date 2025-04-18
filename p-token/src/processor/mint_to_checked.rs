use {
    super::shared,
    pinocchio::{account_info::AccountInfo, ProgramResult},
    spl_token_interface::error::TokenError,
};

#[inline(always)]
pub fn process_mint_to_checked(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    // expected u64 (8) + u8 (1)
    let (amount, decimals) = if instruction_data.len() == 9 {
        let (amount, decimals) = instruction_data.split_at(core::mem::size_of::<u64>());
        (
            u64::from_le_bytes(amount.try_into().unwrap()),
            decimals.first().copied(),
        )
    } else {
        return Err(TokenError::InvalidInstruction.into());
    };

    shared::mint_to::process_mint_to(accounts, amount, decimals)
}
