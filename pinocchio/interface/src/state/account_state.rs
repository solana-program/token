use pinocchio::program_error::ProgramError;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AccountState {
    /// Account is not yet initialized
    Uninitialized,

    /// Account is initialized; the account owner and/or delegate may perform
    /// permitted operations on this account
    Initialized,

    /// Account has been frozen by the mint freeze authority. Neither the
    /// account owner nor the delegate are able to perform operations on
    /// this account.
    Frozen,
}

impl TryFrom<u8> for AccountState {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            // SAFETY: `value` is guaranteed to be in the range of the enum variants.
            0..=2 => Ok(unsafe { core::mem::transmute::<u8, AccountState>(value) }),
            _ => Err(ProgramError::InvalidAccountData),
        }
    }
}
