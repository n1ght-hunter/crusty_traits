//! # Crusty Traits Core
//!
//! This module provides core traits and types for creating C-compatible vtables for Rust traits.
//! It includes the fundamental building blocks: `CRef`, `CRefMut`, `CRepr`, `CDrop`, and `AsVTable`.
//!
//! ## Core Types
//!
//! - [`CRef`] - A C-compatible reference to a trait object
//! - [`CRefMut`] - A C-compatible mutable reference to a trait object  
//! - [`CRepr`] - A C-compatible representation of a trait object with its vtable
//! - [`CDrop`] - A trait for dropping objects in a C-compatible way
//! - [`AsVTable`] - A trait for converting types to vtables
//!
//! These types work together to enable safe FFI interactions with Rust trait objects.

mod trait_wrapper;

pub use trait_wrapper::*;

/// A trait that represents dropping a Rust object in a C-compatible way.
pub trait CDrop {
    /// Drops the object represented by the given `CRepr`.
    fn drop(repr: CRefMut<Self>);
}

/// A trait that provides a way to convert a type into a C-compatible vtable.
pub trait AsVTable<T: ?Sized> {
    /// Return a vtable for the type.
    fn as_vtable(&self) -> T;
}
