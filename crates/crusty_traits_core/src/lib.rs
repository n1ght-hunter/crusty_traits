//! This module provides core traits and types for creating C-compatible vtables for Rust traits.
//! It includes the `CRef`, `CRefMut`, and `CDrop` traits

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
