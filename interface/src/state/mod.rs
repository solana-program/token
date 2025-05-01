use pinocchio::{program_error::ProgramError, ProgramResult};

pub mod account;
pub mod account_state;
pub mod mint;
pub mod multisig;

/// Type alias for fields represented as `COption`.
pub type COption<T> = ([u8; 4], T);

/// Marker trait for types that can be cast from a raw pointer.
///
/// It is up to the type implementing this trait to guarantee that the cast is
/// safe, i.e., the fields of the type are well aligned and there are no padding
/// bytes.
pub trait Transmutable {
    /// The length of the type.
    ///
    /// This must be equal to the size of each individual field in the type.
    const LEN: usize;
}

/// Trait to represent a type that can be initialized.
///
/// Types implementing this trait must provide a method to check if the object
/// is initialized, i.e., if all required fields are set to valid values and
/// they represent an initialized state.
pub trait Initializable {
    /// Return `true` if the object is initialized.
    fn is_initialized(&self) -> Result<bool, ProgramError>;
}

/// Return a reference for an initialized `T` from the given bytes.
///
/// # Safety
///
/// The caller must ensure that `bytes` contains a valid representation of `T`.
#[inline(always)]
pub unsafe fn load<T: Initializable + Transmutable>(bytes: &[u8]) -> Result<&T, ProgramError> {
    load_unchecked(bytes).and_then(|t: &T| {
        // checks if the data is initialized
        if t.is_initialized()? {
            Ok(t)
        } else {
            Err(ProgramError::UninitializedAccount)
        }
    })
}

/// Return a `T` reference from the given bytes.
///
/// This function does not check if the data is initialized.
///
/// # Safety
///
/// The caller must ensure that `bytes` contains a valid representation of `T`.
#[inline(always)]
pub unsafe fn load_unchecked<T: Transmutable>(bytes: &[u8]) -> Result<&T, ProgramError> {
    if bytes.len() != T::LEN {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(&*(bytes.as_ptr() as *const T))
}

/// Return a mutable reference for an initialized `T` from the given bytes.
///
/// # Safety
///
/// The caller must ensure that `bytes` contains a valid representation of `T`.
#[inline(always)]
pub unsafe fn load_mut<T: Initializable + Transmutable>(
    bytes: &mut [u8],
) -> Result<&mut T, ProgramError> {
    load_mut_unchecked(bytes).and_then(|t: &mut T| {
        // checks if the data is initialized
        if t.is_initialized()? {
            Ok(t)
        } else {
            Err(ProgramError::UninitializedAccount)
        }
    })
}

/// Return a mutable `T` reference from the given bytes.
///
/// This function does not check if the data is initialized.
///
/// # Safety
///
/// The caller must ensure that `bytes` contains a valid representation of `T`.
#[inline(always)]
pub unsafe fn load_mut_unchecked<T: Transmutable>(
    bytes: &mut [u8],
) -> Result<&mut T, ProgramError> {
    if bytes.len() != T::LEN {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(&mut *(bytes.as_mut_ptr() as *mut T))
}

/// Validates a `COption` mask value.
#[inline(always)]
const fn validate_option(value: [u8; 4]) -> ProgramResult {
    if u32::from_le_bytes(value) > 1 {
        Err(ProgramError::InvalidAccountData)
    } else {
        Ok(())
    }
}
