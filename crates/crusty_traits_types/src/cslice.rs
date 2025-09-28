//! C-compatible slice types and traits. that converts to the rust slice type.
//!

use crusty_traits_core::*;
use crusty_traits_macros::crusty_trait;

#[crusty_trait]
/// A trait that represents a C-compatible slice.
pub trait CSlice<V> {
    /// Returns a pointer to the first element of the slice.
    fn as_ptr(&self) -> *const V;
    /// Returns the length of the slice.
    fn len(&self) -> usize;
}

/// Extension methods for the `CSlice` trait.
pub trait CSliceExt<V>: CSlice<V> {
    /// Returns the slice as a Rust slice.
    fn as_slice(&self) -> &[V] {
        #[allow(unsafe_code)]
        unsafe {
            std::slice::from_raw_parts(self.as_ptr(), self.len())
        }
    }

    /// Returns the slice as a mutable Rust slice.
    fn as_mut_slice(&mut self) -> &mut [V] {
        #[allow(unsafe_code)]
        unsafe {
            std::slice::from_raw_parts_mut(self.as_ptr() as *mut V, self.len())
        }
    }
}

impl<T, V> CSliceExt<V> for T where T: CSlice<V> {}

impl<V> CSlice<V> for [V] {
    fn as_ptr(&self) -> *const V {
        self.as_ptr()
    }

    fn len(&self) -> usize {
        self.len()
    }
}

impl<V> CSlice<V> for Vec<V> {
    fn as_ptr(&self) -> *const V {
        self.as_ptr()
    }

    fn len(&self) -> usize {
        self.len()
    }
}
