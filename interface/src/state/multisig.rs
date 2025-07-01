use {
    super::{Initializable, Transmutable},
    pinocchio::{program_error::ProgramError, pubkey::Pubkey},
};

/// Minimum number of multisignature signers (min N)
pub const MIN_SIGNERS: u8 = 1;

/// Maximum number of multisignature signers (max N)
pub const MAX_SIGNERS: u8 = 11;

/// Multisignature data.
#[repr(C)]
pub struct Multisig {
    /// Number of signers required.
    pub m: u8,

    /// Number of valid signers.
    pub n: u8,

    /// Is `true` if this structure has been initialized.
    is_initialized: u8,

    /// Signer public keys.
    pub signers: [Pubkey; MAX_SIGNERS as usize],
}

impl Multisig {
    /// Utility function that checks index is between [`MIN_SIGNERS`] and
    /// [`MAX_SIGNERS`].
    pub fn is_valid_signer_index(index: u8) -> bool {
        (MIN_SIGNERS..=MAX_SIGNERS).contains(&index)
    }

    #[inline]
    pub fn set_initialized(&mut self, value: bool) {
        self.is_initialized = value as u8;
    }
}

unsafe impl Transmutable for Multisig {
    /// The length of the `Multisig` account data.
    const LEN: usize = core::mem::size_of::<Multisig>();
}

impl Initializable for Multisig {
    #[inline(always)]
    fn is_initialized(&self) -> Result<bool, ProgramError> {
        match self.is_initialized {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(ProgramError::InvalidAccountData),
        }
    }
}
