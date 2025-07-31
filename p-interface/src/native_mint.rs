//! The Mint that represents the native token.

use pinocchio::pubkey::Pubkey;

/// There are `10^9` lamports in one SOL
pub const DECIMALS: u8 = 9;

// The Mint for native SOL Token accounts
pub const ID: Pubkey = pinocchio_pubkey::pubkey!("So11111111111111111111111111111111111111112");

#[inline(always)]
pub fn is_native_mint(mint: &Pubkey) -> bool {
    mint == &ID
}
