//! # Crusty Traits Types
//!
//! This crate provides C-compatible data types that can be safely used across FFI boundaries
//! alongside the crusty traits system.
//!
//! ## Available Types
//!
//! - [`cslice`] - C-compatible slice types for passing array data across FFI
//! - [`cvec`] - C-compatible vector types for dynamic arrays
//!
//! These types are designed to work seamlessly with the `crusty_trait` macro system
//! and provide safe, efficient data exchange between Rust and C code.

pub mod cslice;
pub mod cvec;
