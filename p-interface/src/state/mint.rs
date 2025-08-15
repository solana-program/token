use {
    super::{COption, Initializable, Transmutable},
    pinocchio::{program_error::ProgramError, pubkey::Pubkey},
};

/// Internal representation of a mint data.
#[repr(C)]
pub struct Mint {
    /// Optional authority used to mint new tokens. The mint authority may only
    /// be provided during mint creation. If no mint authority is present
    /// then the mint has a fixed supply and no further tokens may be
    /// minted.
    mint_authority: COption<Pubkey>,

    /// Total supply of tokens.
    supply: [u8; 8],

    /// Number of base 10 digits to the right of the decimal place.
    pub decimals: u8,

    /// Is `true` if this structure has been initialized.
    is_initialized: u8,

    // Indicates whether the freeze authority is present or not.
    //freeze_authority_option: [u8; 4],
    /// Optional authority to freeze token accounts.
    freeze_authority: COption<Pubkey>,
}

impl Mint {
    #[inline(always)]
    pub fn set_supply(&mut self, supply: u64) {
        self.supply = supply.to_le_bytes();
    }

    #[inline(always)]
    pub fn supply(&self) -> u64 {
        u64::from_le_bytes(self.supply)
    }

    #[inline(always)]
    pub fn set_initialized(&mut self) {
        self.is_initialized = 1;
    }

    #[inline(always)]
    pub fn clear_mint_authority(&mut self) {
        self.mint_authority.0[0] = 0;
    }

    #[inline(always)]
    pub fn set_mint_authority(&mut self, mint_authority: &Pubkey) {
        self.mint_authority.0[0] = 1;
        self.mint_authority.1 = *mint_authority;
    }

    #[inline(always)]
    pub fn mint_authority(&self) -> Option<&Pubkey> {
        if self.mint_authority.0[0] == 1 {
            Some(&self.mint_authority.1)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn clear_freeze_authority(&mut self) {
        self.freeze_authority.0[0] = 0;
    }

    #[inline(always)]
    pub fn set_freeze_authority(&mut self, freeze_authority: &Pubkey) {
        self.freeze_authority.0[0] = 1;
        self.freeze_authority.1 = *freeze_authority;
    }

    #[inline(always)]
    pub fn freeze_authority(&self) -> Option<&Pubkey> {
        if self.freeze_authority.0[0] == 1 {
            Some(&self.freeze_authority.1)
        } else {
            None
        }
    }
}

unsafe impl Transmutable for Mint {
    /// The length of the `Mint` account data.
    const LEN: usize = core::mem::size_of::<Mint>();
}

impl Initializable for Mint {
    #[inline(always)]
    fn is_initialized(&self) -> Result<bool, ProgramError> {
        match self.is_initialized {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(ProgramError::InvalidAccountData),
        }
    }
}
