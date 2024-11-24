use pinocchio::{
    account_info::{AccountInfo, Ref},
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::program::ID;

use super::COption;

/// Internal representation of a mint data.
#[repr(C)]
pub struct Mint {
    /// Optional authority used to mint new tokens. The mint authority may only
    /// be provided during mint creation. If no mint authority is present
    /// then the mint has a fixed supply and no further tokens may be
    /// minted.
    pub mint_authority: COption<Pubkey>,

    /// Total supply of tokens.
    supply: [u8; 8],

    /// Number of base 10 digits to the right of the decimal place.
    pub decimals: u8,

    /// Is `true` if this structure has been initialized.
    is_initialized: u8,

    // Indicates whether the freeze authority is present or not.
    //freeze_authority_option: [u8; 4],
    /// Optional authority to freeze token accounts.
    pub freeze_authority: COption<Pubkey>,
}

impl Mint {
    /// The length of the `Mint` account data.
    pub const LEN: usize = core::mem::size_of::<Mint>();

    /// Return a `Mint` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline]
    pub fn from_account_info(account_info: &AccountInfo) -> Result<Ref<Mint>, ProgramError> {
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

    /// Return a `Mint` from the given account info.
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

    /// Return a `Mint` reference from the given bytes.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` contains a valid representation of `Mint`.
    #[inline]
    pub unsafe fn from_bytes(bytes: &[u8]) -> &Self {
        &*(bytes.as_ptr() as *const Mint)
    }

    /// Return a mutable `Mint` reference from the given bytes.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` contains a valid representation of `Mint`.
    #[inline]
    pub unsafe fn from_bytes_mut(bytes: &mut [u8]) -> &mut Self {
        &mut *(bytes.as_mut_ptr() as *mut Mint)
    }

    #[inline]
    pub fn set_supply(&mut self, supply: u64) {
        self.supply = supply.to_le_bytes();
    }

    #[inline]
    pub fn supply(&self) -> u64 {
        u64::from_le_bytes(self.supply)
    }

    #[inline]
    pub fn set_initialized(&mut self, value: bool) {
        self.is_initialized = value as u8;
    }

    #[inline]
    pub fn is_initialized(&self) -> bool {
        self.is_initialized == 1
    }

    #[inline]
    pub fn clear_mint_authority(&mut self) {
        self.mint_authority.0[0] = 0;
    }

    #[inline]
    pub fn set_mint_authority(&mut self, mint_authority: &Pubkey) {
        self.mint_authority.0[0] = 1;
        self.mint_authority.1 = *mint_authority;
    }

    #[inline]
    pub fn mint_authority(&self) -> Option<&Pubkey> {
        if self.mint_authority.0[0] == 1 {
            Some(&self.mint_authority.1)
        } else {
            None
        }
    }

    #[inline]
    pub fn clear_freeze_authority(&mut self) {
        self.freeze_authority.0[0] = 0;
    }

    #[inline]
    pub fn set_freeze_authority(&mut self, freeze_authority: &Pubkey) {
        self.freeze_authority.0[0] = 1;
        self.freeze_authority.1 = *freeze_authority;
    }

    #[inline]
    pub fn freeze_authority(&self) -> Option<&Pubkey> {
        if self.freeze_authority.0[0] == 1 {
            Some(&self.freeze_authority.1)
        } else {
            None
        }
    }
}
