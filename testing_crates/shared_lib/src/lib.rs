//! This is a shared library that can be used in C or Rust.
#![allow(unsafe_code)]
use crusty_traits::prelude::*;
pub use crusty_traits::types::cvec::CVecVTable;

/// print "Hello from the shared library!"
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hello_world() {
    println!("Hello from the rust shared library!");
}

/// add two numbers and return the result
#[unsafe(no_mangle)]
pub unsafe extern "C" fn add(a: isize, b: isize) -> isize {
    a + b
}

/// multiply two numbers and return the result
#[unsafe(no_mangle)]
pub unsafe extern "C" fn multiply(a: isize, b: isize) -> isize {
    a * b
}

#[unsafe(no_mangle)]
/// Create a new C-compatible vector of i32 and return it.
pub unsafe extern "C" fn create_vector() -> CRepr<CVecVTable<i32>> {
    let vec: Vec<i32> = Vec::new();
    CVecVTable::new_boxed(vec)
}
