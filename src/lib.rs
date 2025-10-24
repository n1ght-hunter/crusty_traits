//! # Crusty Traits
//! ## C <-> Rust Traits
//!
//! A crate that creates a macro and supporting code to allow for traits to be FFI-safe using C ABI.
//!
//! > **Warning**: This crate uses unsafe code and may be unsound if used incorrectly. Use at your own risk.
//! > If any issues are found please open an issue or a PR.
//!
//! ## Usage
//!
//! Add the following to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! crusty_traits = "0.1"
//! ```
//!
//! Then in your code:
//!
//! ```rust
//! use crusty_traits::prelude::*;
//!
//! #[crusty_trait]
//! pub trait MyTrait {
//!     fn method1(&self);
//!     fn method2(&mut self, value: i32) -> i32;
//! }
//! ```
//!
//! ## Crate Details
//!
//! This crate provides a macro `crusty_trait` that generates the necessary boilerplate code to create
//! a C-compatible vtable for a given Rust trait. This allows Rust traits to be used across FFI boundaries,
//! making it easier to use Rust shared libraries or plugins in C or other languages that can interface with C.
//!
//! Each trait that is annotated with `crusty_trait` will have a corresponding vtable struct generated,
//! along with implementations for `CRepr` and `CDrop` to manage the memory and lifecycle of the trait objects.
//! The generated vtable struct will contain function pointers for each method in the trait, as well as a
//! drop function to properly clean up the trait object when it is no longer needed.
//!
//! The trait is also implemented for `CRepr<MyTraitVTable>` and any `CRepr<GEN>` where `GEN` implements
//! `AsVTable<&'static MyTraitVTable>` (used for super/sub traits) and `CDrop`, allowing for seamless
//! usage of the trait across FFI boundaries in Rust code.

pub use crusty_traits_core::*;
pub use crusty_traits_macros::crusty_trait;

/// Prelude module that exports all the necessary types and traits for creating FFI-safe traits.
///
/// This module provides a convenient way to import all the core functionality needed to use
/// the `crusty_trait` macro and work with C-compatible vtables.
pub mod prelude {
    pub use crate::AsVTable;
    pub use crate::CDrop;
    pub use crate::CRef;
    pub use crate::CRefMut;
    pub use crate::CRepr;
    pub use crate::crusty_trait;
}

pub use crusty_traits_types as types;

#[cfg(test)]
mod tests {
    use super::*;

    #[crusty_trait]
    trait Buffer: Send + Sync {
        fn as_slice(&self) -> *mut u8;
        fn extend(&mut self, amount: usize);
        fn len(&self) -> usize;
    }

    impl Buffer for Vec<u8> {
        fn as_slice(&self) -> *mut u8 {
            self.as_ptr() as *mut u8
        }

        fn extend(&mut self, amount: usize) {
            self.extend_from_slice(&vec![0; amount]);
        }

        fn len(&self) -> usize {
            self.len()
        }
    }

    #[test]
    fn test_crusty_trait() {
        let mut buffer = Vec::new();
        Buffer::extend(&mut buffer, 10);
        assert_eq!(buffer.len(), 10);
    }

    #[test]
    fn test_c_repr() {
        let buffer = Vec::new();
        let mut vtable = BufferVTable::new_boxed(buffer);
        vtable.extend(10);
        assert_eq!(vtable.len(), 10);
        let slice = vtable.as_slice();
        assert!(!slice.is_null());
        #[allow(unsafe_code)]
        {
            let slice = unsafe { std::slice::from_raw_parts_mut(slice as *mut _, vtable.len()) };
            assert_eq!(slice.len(), 10);
            for i in 0..10 {
                slice[i] = i as u8;
            }
        }
        let slice = vtable.as_slice();
        #[allow(unsafe_code)]
        {
            let slice = unsafe { std::slice::from_raw_parts(slice as *const _, vtable.len()) };
            assert_eq!(slice.len(), 10);
            for i in 0..10 {
                assert_eq!(slice[i], i as u8);
            }
        }
    }
}
