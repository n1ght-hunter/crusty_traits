# crusty_traits_core

<div align="center">

[![Documentation](https://docs.rs/crusty_traits_core/badge.svg)](https://docs.rs/crusty_traits_core)
[![Crates.io](https://img.shields.io/crates/v/crusty_traits_core.svg)](https://crates.io/crates/crusty_traits_core)
[![License](https://img.shields.io/crates/l/crusty_traits_core.svg)](https://github.com/n1ght-hunter/crusty_traits/blob/master/LICENSE)
[![Downloads](https://img.shields.io/crates/d/crusty_traits_core.svg)](https://crates.io/crates/crusty_traits_core)

</div>

Core types and traits for crusty_traits - FFI-safe trait objects.

## Overview

This crate provides the fundamental types and traits that enable FFI-safe trait objects in Rust. It contains the core infrastructure that allows Rust traits to be safely used across C ABI boundaries.

## Key Components

- **`CRepr<T>`**: A C-compatible representation wrapper for trait objects
- **`CRef<T>` and `CRefMut<T>`**: Safe references to C-compatible objects
- **`CDrop`**: Trait for proper cleanup of C-compatible objects
- **`AsVTable<T>`**: Trait for vtable conversion and management

## Usage

This crate is typically used indirectly through the main `crusty_traits` crate:

```toml
[dependencies]
crusty_traits = "0.1"
```

For direct usage:

```toml
[dependencies]
crusty_traits_core = "0.1"
```

## Documentation

See the [API documentation](https://docs.rs/crusty_traits_core) for detailed usage information.
