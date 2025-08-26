#![no_std]

pub mod error;
pub mod instruction;
pub mod native_mint;
pub mod state;

pub mod program {
    pinocchio_pubkey::declare_id!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
}

/// A "dummy" function with a hint to the compiler that it is unlikely to be
/// called.
///
/// This function is used as a hint to the compiler to optimize other code paths
/// instead of the one where the function is used.
#[cold]
pub const fn cold_path() {}

/// Return the given `bool` value with a hint to the compiler that `true` is the
/// likely case.
#[inline(always)]
pub const fn likely(b: bool) -> bool {
    if b {
        true
    } else {
        cold_path();
        false
    }
}

/// Return a given `bool` value with a hint to the compiler that `false` is the
/// likely case.
#[inline(always)]
pub const fn unlikely(b: bool) -> bool {
    if b {
        cold_path();
        true
    } else {
        false
    }
}
