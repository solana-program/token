//! Error types

use pinocchio::program_error::ProgramError;

/// Errors that may be returned by the Token program.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TokenError {
    // 0
    /// Lamport balance below rent-exempt threshold.
    NotRentExempt,
    /// Insufficient funds for the operation requested.
    InsufficientFunds,
    /// Invalid Mint.
    InvalidMint,
    /// Account not associated with this Mint.
    MintMismatch,
    /// Owner does not match.
    OwnerMismatch,

    // 5
    /// This token's supply is fixed and new tokens cannot be minted.
    FixedSupply,
    /// The account cannot be initialized because it is already being used.
    AlreadyInUse,
    /// Invalid number of provided signers.
    InvalidNumberOfProvidedSigners,
    /// Invalid number of required signers.
    InvalidNumberOfRequiredSigners,
    /// State is uninitialized.
    UninitializedState,

    // 10
    /// Instruction does not support native tokens
    NativeNotSupported,
    /// Non-native account can only be closed if its balance is zero
    NonNativeHasBalance,
    /// Invalid instruction
    InvalidInstruction,
    /// State is invalid for requested operation.
    InvalidState,
    /// Operation overflowed
    Overflow,

    // 15
    /// Account does not support specified authority type.
    AuthorityTypeNotSupported,
    /// This token mint cannot freeze accounts.
    MintCannotFreeze,
    /// Account is frozen; all account operations will fail
    AccountFrozen,
    /// Mint decimals mismatch between the client and mint
    MintDecimalsMismatch,
    /// Instruction does not support non-native tokens
    NonNativeNotSupported,
}

impl From<TokenError> for ProgramError {
    fn from(e: TokenError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
