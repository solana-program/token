use {
    super::{shared, U64_BYTES},
    pinocchio::{account_info::AccountInfo, ProgramResult},
    spl_token_interface::error::TokenError,
};

#[inline(always)]
pub fn process_mint_to(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let amount = if instruction_data.len() >= U64_BYTES {
        // SAFETY: The minimum size of the instruction data is `U64_BYTES` bytes.
        unsafe { u64::from_le_bytes(*(instruction_data.as_ptr() as *const [u8; U64_BYTES])) }
    } else {
        return Err(TokenError::InvalidInstruction.into());
    };

    shared::mint_to::process_mint_to(accounts, amount, None)
}
