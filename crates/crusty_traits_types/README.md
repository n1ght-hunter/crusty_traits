# crusty_traits_types

<div align="center">

[![Documentation](https://docs.rs/crusty_traits_types/badge.svg)](https://docs.rs/crusty_traits_types)
[![Crates.io](https://img.shields.io/crates/v/crusty_traits_types.svg)](https://crates.io/crates/crusty_traits_types)
[![License](https://img.shields.io/crates/l/crusty_traits_types.svg)](https://github.com/n1ght-hunter/crusty_traits/blob/master/LICENSE)
[![Downloads](https://img.shields.io/crates/d/crusty_traits_types.svg)](https://crates.io/crates/crusty_traits_types)

</div>

C-compatible types for crusty_traits - FFI-safe vectors and slices.

## Overview

This crate provides C-compatible versions of common Rust data structures like `Vec` and slice types. These types can be safely passed across FFI boundaries while maintaining memory safety.

## Key Types

- **`CVec<T>`**: A C-compatible vector that can be safely used across FFI boundaries
- **`CSlice<T>`**: A C-compatible slice representation
- Additional utility types for FFI interoperability

## Features

- Memory-safe FFI data structures
- Serde serialization support
- Zero-copy conversions where possible
- Proper cleanup and memory management

## Usage

This crate is typically used through the main `crusty_traits` crate:

```toml
[dependencies]
crusty_traits = "0.1"
```

For direct usage:

```toml
[dependencies]
crusty_traits_types = "0.1"
```

```rust
use crusty_traits_types::CVec;

// Create a C-compatible vector
let cvec = CVec::from(vec![1, 2, 3, 4]);
```

## Documentation

See the [API documentation](https://docs.rs/crusty_traits_types) for detailed usage information.
