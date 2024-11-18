use bytemuck::{Pod, Zeroable};
use pinocchio::pubkey::Pubkey;

use super::{PodBool, PodCOption, PodU64};

/// Mint data.
#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
pub struct Mint {
    /// Optional authority used to mint new tokens. The mint authority may only
    /// be provided during mint creation. If no mint authority is present
    /// then the mint has a fixed supply and no further tokens may be
    /// minted.
    pub mint_authority: PodCOption<Pubkey>,

    /// Total supply of tokens.
    pub supply: PodU64,

    /// Number of base 10 digits to the right of the decimal place.
    pub decimals: u8,

    /// Is `true` if this structure has been initialized
    pub is_initialized: PodBool,

    /// Optional authority to freeze token accounts.
    pub freeze_authority: PodCOption<Pubkey>,
}

impl Mint {
    /// Size of the `Mint` account.
    pub const LEN: usize = core::mem::size_of::<Self>();
}
