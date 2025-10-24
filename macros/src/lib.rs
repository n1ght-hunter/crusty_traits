//! # Crusty Traits Macros
//!
//! This crate provides the `crusty_trait` procedural macro that transforms regular Rust traits
//! into FFI-safe traits with C-compatible vtables.
//!
//! ## Usage
//!
//! Apply the `#[crusty_trait]` attribute to any trait to generate the necessary boilerplate
//! for FFI compatibility:
//!
//! ```rust,ignore
//! use crusty_traits::prelude::*;
//!
//! #[crusty_trait]
//! pub trait MyTrait {
//!     fn method(&self, value: i32) -> i32;
//! }
//! ```
//!
//! The macro will generate:
//! - A C-compatible vtable struct (`MyTraitVTable`)
//! - Implementations for `CRepr<MyTraitVTable>`
//! - Memory management through `CDrop`
//! - Helper methods for creating and managing trait objects

use crusty_trait_macro::impl_crusty_trait;
use quote::ToTokens;
use syn::ItemTrait;

/// The `crusty_trait` procedural macro transforms a Rust trait into an FFI-safe trait
/// with a C-compatible vtable.
///
/// This macro generates:
/// - A repr(C) vtable struct containing function pointers for each trait method
/// - Implementation of the original trait for `CRepr<TraitVTable>`  
/// - Memory management through the `CDrop` trait
/// - Helper functions for creating and managing trait objects across FFI boundaries
///
/// # Example
///
/// ```rust,ignore
/// use crusty_traits::prelude::*;
///
/// #[crusty_trait]
/// pub trait Buffer {
///     fn len(&self) -> usize;
///     fn extend(&mut self, amount: usize);
/// }
/// ```
///
/// This will generate a `BufferVTable` struct and all necessary implementations
/// to use this trait safely across FFI boundaries.
///
/// # Safety
///
/// The generated code uses unsafe operations internally but provides a safe API.
/// However, care must be taken when using the trait objects across FFI boundaries
/// to ensure proper lifetime management and memory safety.
#[proc_macro_attribute]
pub fn crusty_trait(
    _attrs: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as ItemTrait);

    impl_crusty_trait(input).to_token_stream().into()
}
