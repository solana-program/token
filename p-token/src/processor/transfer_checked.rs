use {
    super::{shared, U64_BYTES},
    pinocchio::{account_info::AccountInfo, ProgramResult},
    spl_token_interface::error::TokenError,
};

#[inline(always)]
pub fn process_transfer_checked(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // expected u64 (8) + u8 (1)
    let (amount, decimals) = if instruction_data.len() >= 9 {
        let (amount, decimals) = instruction_data.split_at(U64_BYTES);
        (
            // SAFETY: The size of `amount` is `U64_BYTES` bytes.
            unsafe { u64::from_le_bytes(*(amount.as_ptr() as *const [u8; U64_BYTES])) },
            decimals.first().copied(),
        )
    } else {
        return Err(TokenError::InvalidInstruction.into());
    };

    shared::transfer::process_transfer(accounts, amount, decimals)
}
