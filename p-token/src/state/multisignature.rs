use bytemuck::{Pod, Zeroable};
use pinocchio::pubkey::Pubkey;

use super::PodBool;

/// Minimum number of multisignature signers (min N)
pub const MIN_SIGNERS: usize = 1;
/// Maximum number of multisignature signers (max N)
pub const MAX_SIGNERS: usize = 11;

/// Multisignature data.
#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
pub struct Multisig {
    /// Number of signers required
    pub m: u8,
    /// Number of valid signers
    pub n: u8,
    /// Is `true` if this structure has been initialized
    pub is_initialized: PodBool,
    /// Signer public keys
    pub signers: [Pubkey; MAX_SIGNERS],
}

impl Multisig {
    pub const LEN: usize = core::mem::size_of::<Multisig>();
}
