use std::mem::align_of;

use bytemuck::{Pod, Zeroable};

pub mod account;
pub mod mint;
pub mod multisignature;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PodCOption<T: Default + PartialEq + Pod + Sized> {
    /// Indicates if the option is `Some` or `None`.
    tag: [u8; 4],

    /// The value of the option.
    value: T,
}

impl<T: Default + PartialEq + Pod + Sized> From<Option<T>> for PodCOption<T> {
    fn from(value: Option<T>) -> Self {
        if align_of::<T>() != 1 {
            panic!("PodCOption only supports Pod types with alignment 1");
        }

        match value {
            Some(value) => Self {
                tag: [1, 0, 0, 0],
                value,
            },
            None => Self {
                tag: [0, 0, 0, 0],
                value: T::default(),
            },
        }
    }
}

impl<T: Default + PartialEq + Pod + Sized> PodCOption<T> {
    pub const NONE: [u8; 4] = [0, 0, 0, 0];

    pub const SOME: [u8; 4] = [1, 0, 0, 0];

    /// Returns `true` if the option is a `None` value.
    #[inline]
    pub fn is_none(&self) -> bool {
        self.tag == Self::NONE
    }

    /// Returns `true` if the option is a `Some` value.
    #[inline]
    pub fn is_some(&self) -> bool {
        !self.is_none()
    }

    /// Returns the contained value as an `Option`.
    #[inline]
    pub fn get(self) -> Option<T> {
        if self.is_none() {
            None
        } else {
            Some(self.value)
        }
    }

    /// Returns the contained value as an `Option`.
    #[inline]
    pub fn as_ref(&self) -> Option<&T> {
        if self.is_none() {
            None
        } else {
            Some(&self.value)
        }
    }

    /// Returns the contained value as a mutable `Option`.
    #[inline]
    pub fn as_mut(&mut self) -> Option<&mut T> {
        if self.is_none() {
            None
        } else {
            Some(&mut self.value)
        }
    }

    #[inline]
    pub fn set(&mut self, value: T) {
        self.tag = Self::SOME;
        self.value = value;
    }

    #[inline]
    pub fn clear(&mut self) {
        self.tag = Self::NONE;
        // we don't need to zero the value since the tag
        // indicates it is a `None` value
    }
}

/// ## Safety
///
/// `PodCOption` requires a `Pod` type `T` with alignment of 1.
unsafe impl<T: Default + PartialEq + Pod + Sized> Pod for PodCOption<T> {}

/// ## Safety
///
/// `PodCOption` requires a `Pod` type `T` with alignment of 1.
unsafe impl<T: Default + PartialEq + Pod + Sized> Zeroable for PodCOption<T> {}

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
pub struct PodBool(u8);

impl From<bool> for PodBool {
    fn from(b: bool) -> Self {
        Self(b.into())
    }
}

impl From<&bool> for PodBool {
    fn from(b: &bool) -> Self {
        Self((*b).into())
    }
}

impl From<&PodBool> for bool {
    fn from(b: &PodBool) -> Self {
        b.0 != 0
    }
}

impl From<PodBool> for bool {
    fn from(b: PodBool) -> Self {
        b.0 != 0
    }
}
