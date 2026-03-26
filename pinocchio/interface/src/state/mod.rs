use pinocchio::{
    hint::{likely, unlikely},
    program_error::ProgramError,
};

pub mod account;
pub mod account_state;
pub mod mint;
pub mod multisig;

/// Type alias for fields represented as `COption`.
pub type COption<T> = ([u8; 4], T);

/// Marker trait for types that can be cast from a raw pointer.
///
/// # Safety
///
/// It is up to the type implementing this trait to guarantee that the cast is
/// safe, i.e., the fields of the type are well aligned and there are no padding
/// bytes.
pub unsafe trait Transmutable: sealed::Sealed {
    /// The length of the type.
    ///
    /// This must be equal to the size of each individual field in the type.
    const LEN: usize;
}

/// Trait to represent a type that can be initialized.
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
        if likely(t.is_initialized()?) {
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
    const {
        assert!(
            core::mem::align_of::<T>() == 1,
            "<T> must have minimum alignment of 1"
        );
    };

    if unlikely(bytes.len() != T::LEN) {
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
        if likely(t.is_initialized()?) {
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
    const {
        assert!(
            core::mem::align_of::<T>() == 1,
            "<T> must have minimum alignment of 1"
        );
    };

    if unlikely(bytes.len() != T::LEN) {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(&mut *(bytes.as_mut_ptr() as *mut T))
}

/// Private module to seal the `Transmutable` trait.
mod sealed {
    /// Sealed trait to prevent external implementation of `Transmutable`.
    pub trait Sealed {}
}
