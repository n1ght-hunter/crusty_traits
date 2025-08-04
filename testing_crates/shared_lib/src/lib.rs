//! This is a shared library that can be used in C or Rust.
#![allow(unsafe_code)]

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