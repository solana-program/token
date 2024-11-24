use pinocchio::{
    account_info::{AccountInfo, Ref},
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::program::ID;

use super::{account_state::AccountState, COption};

/// Internal representation of a token account data.
#[repr(C)]
pub struct Account {
    /// The mint associated with this account
    pub mint: Pubkey,

    /// The owner of this account.
    pub owner: Pubkey,

    /// The amount of tokens this account holds.
    amount: [u8; 8],

    /// If `delegate` is `Some` then `delegated_amount` represents
    /// the amount authorized by the delegate.
    delegate: COption<Pubkey>,

    /// The account's state.
    pub state: AccountState,

    /// Indicates whether this account represents a native token or not.
    is_native: [u8; 4],

    /// If is_native.is_some, this is a native token, and the value logs the
    /// rent-exempt reserve. An Account is required to be rent-exempt, so
    /// the value is used by the Processor to ensure that wrapped SOL
    /// accounts do not drop below this threshold.
    native_amount: [u8; 8],

    /// The amount delegated.
    delegated_amount: [u8; 8],

    /// Optional authority to close the account.
    close_authority: COption<Pubkey>,
}

impl Account {
    pub const LEN: usize = core::mem::size_of::<Account>();

    /// Return a `TokenAccount` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline]
    pub fn from_account_info(account_info: &AccountInfo) -> Result<Ref<Account>, ProgramError> {
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        if account_info.owner() != &ID {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(Ref::map(account_info.try_borrow_data()?, |data| unsafe {
            Self::from_bytes(data)
        }))
    }

    /// Return a `TokenAccount` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, but does not
    /// perform the borrow check.
    ///
    /// # Safety
    ///
    /// The caller must ensure that it is safe to borrow the account data – e.g., there are
    /// no mutable borrows of the account data.
    #[inline]
    pub unsafe fn from_account_info_unchecked(
        account_info: &AccountInfo,
    ) -> Result<&Account, ProgramError> {
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        if account_info.owner() != &ID {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(Self::from_bytes(account_info.borrow_data_unchecked()))
    }

    /// Return a `TokenAccount` from the given bytes.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` contains a valid representation of `TokenAccount`.
    #[inline(always)]
    pub unsafe fn from_bytes(bytes: &[u8]) -> &Self {
        &*(bytes.as_ptr() as *const Account)
    }

    /// Return a mutable `Mint` reference from the given bytes.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` contains a valid representation of `Mint`.
    #[inline(always)]
    pub unsafe fn from_bytes_mut(bytes: &mut [u8]) -> &mut Self {
        &mut *(bytes.as_mut_ptr() as *mut Account)
    }

    #[inline]
    pub fn set_amount(&mut self, amount: u64) {
        self.amount = amount.to_le_bytes();
    }

    #[inline]
    pub fn amount(&self) -> u64 {
        u64::from_le_bytes(self.amount)
    }

    #[inline]
    pub fn clear_delegate(&mut self) {
        self.delegate.0[0] = 0;
    }

    #[inline]
    pub fn set_delegate(&mut self, delegate: &Pubkey) {
        self.delegate.0[0] = 1;
        self.delegate.1 = *delegate;
    }

    #[inline]
    pub fn delegate(&self) -> Option<&Pubkey> {
        if self.delegate.0[0] == 1 {
            Some(&self.delegate.1)
        } else {
            None
        }
    }

    #[inline]
    pub fn set_native(&mut self, value: bool) {
        self.is_native[0] = value as u8;
    }

    #[inline]
    pub fn is_native(&self) -> bool {
        self.is_native[0] == 1
    }

    #[inline]
    pub fn native_amount(&self) -> Option<u64> {
        if self.is_native() {
            Some(u64::from_le_bytes(self.native_amount))
        } else {
            None
        }
    }

    #[inline]
    pub fn set_delegated_amount(&mut self, amount: u64) {
        self.delegated_amount = amount.to_le_bytes();
    }

    #[inline]
    pub fn delegated_amount(&self) -> u64 {
        u64::from_le_bytes(self.delegated_amount)
    }

    #[inline]
    pub fn clear_close_authority(&mut self) {
        self.close_authority.0[0] = 0;
    }

    #[inline]
    pub fn set_close_authority(&mut self, value: &Pubkey) {
        self.close_authority.0[0] = 1;
        self.close_authority.1 = *value;
    }

    #[inline]
    pub fn close_authority(&self) -> Option<&Pubkey> {
        if self.close_authority.0[0] == 1 {
            Some(&self.close_authority.1)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn is_initialized(&self) -> bool {
        self.state != AccountState::Uninitialized
    }

    #[inline(always)]
    pub fn is_frozen(&self) -> bool {
        self.state == AccountState::Frozen
    }
}
