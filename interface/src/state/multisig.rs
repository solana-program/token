use pinocchio::{
    account_info::{AccountInfo, Ref},
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::program::ID;

/// Minimum number of multisignature signers (min N)
pub const MIN_SIGNERS: usize = 1;

/// Maximum number of multisignature signers (max N)
pub const MAX_SIGNERS: usize = 11;

/// Multisignature data.
#[repr(C)]
pub struct Multisig {
    /// Number of signers required.
    pub m: u8,

    /// Number of valid signers.
    pub n: u8,

    /// Is `true` if this structure has been initialized
    is_initialized: u8,

    /// Signer public keys
    pub signers: [Pubkey; MAX_SIGNERS],
}

impl Multisig {
    /// The length of the `Multisig` account data.
    pub const LEN: usize = core::mem::size_of::<Multisig>();

    /// Return a `Multisig` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline]
    pub fn from_account_info(account_info: &AccountInfo) -> Result<Ref<Self>, ProgramError> {
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        if account_info.owner() != &ID {
            return Err(ProgramError::InvalidAccountOwner);
        }
        Ok(Ref::map(account_info.try_borrow_data()?, |data| unsafe {
            Self::from_bytes(data)
        }))
    }

    /// Return a `Multisig` from the given account info.
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
    ) -> Result<&Self, ProgramError> {
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        if account_info.owner() != &ID {
            return Err(ProgramError::InvalidAccountOwner);
        }
        Ok(Self::from_bytes(account_info.borrow_data_unchecked()))
    }

    /// Return a `Multisig` reference from the given bytes.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` contains a valid representation of `Multisig`.
    #[inline]
    pub unsafe fn from_bytes(bytes: &[u8]) -> &Self {
        &*(bytes.as_ptr() as *const Multisig)
    }

    /// Return a mutable `Multisig` reference from the given bytes.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` contains a valid representation of `Multisig`.
    #[inline]
    pub unsafe fn from_bytes_mut(bytes: &mut [u8]) -> &mut Self {
        &mut *(bytes.as_mut_ptr() as *mut Multisig)
    }

    /// Utility function that checks index is between [`MIN_SIGNERS`] and [`MAX_SIGNERS`].
    pub fn is_valid_signer_index(index: usize) -> bool {
        (MIN_SIGNERS..=MAX_SIGNERS).contains(&index)
    }

    #[inline]
    pub fn set_initialized(&mut self, value: bool) {
        self.is_initialized = value as u8;
    }

    #[inline]
    pub fn is_initialized(&self) -> bool {
        self.is_initialized == 1
    }
}
