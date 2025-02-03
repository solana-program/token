use pinocchio::program_error::ProgramError;

pub mod account;
pub mod account_state;
pub mod mint;
pub mod multisig;

/// Type alias for fields represented as `COption`.
pub type COption<T> = ([u8; 4], T);

/// Marker trait for types that can cast from a raw pointer.
///
/// It is up to the type implementing this trait to guarantee that the cast is safe,
/// i.e., that the fields of the type are well aligned and there are no padding bytes.
pub trait RawType {
    /// The length of the type.
    ///
    /// This must be equal to the size of each individual field in the type.
    const LEN: usize;
}

/// Trait to represent a type that can be initialized.
pub trait Initializable {
    /// Return `true` if the object is initialized.
    fn is_initialized(&self) -> bool;
}

/// Return a reference for an initialized `T` from the given bytes.
///
/// # Safety
///
/// The caller must ensure that `bytes` contains a valid representation of `T`.
#[inline(always)]
pub unsafe fn load<T: Initializable + RawType>(bytes: &[u8]) -> Result<&T, ProgramError> {
    load_unchecked(bytes).and_then(|t: &T| {
        // checks if the data is initialized
        if t.is_initialized() {
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
pub unsafe fn load_unchecked<T: RawType>(bytes: &[u8]) -> Result<&T, ProgramError> {
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
pub unsafe fn load_mut<T: Initializable + RawType>(
    bytes: &mut [u8],
) -> Result<&mut T, ProgramError> {
    load_mut_unchecked(bytes).and_then(|t: &mut T| {
        // checks if the data is initialized
        if t.is_initialized() {
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
pub unsafe fn load_mut_unchecked<T: RawType>(bytes: &mut [u8]) -> Result<&mut T, ProgramError> {
    if bytes.len() != T::LEN {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(&mut *(bytes.as_mut_ptr() as *mut T))
}
