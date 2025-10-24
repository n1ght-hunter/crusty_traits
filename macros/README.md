# crusty_traits_macros

<div align="center">

[![Documentation](https://docs.rs/crusty_traits_macros/badge.svg)](https://docs.rs/crusty_traits_macros)
[![Crates.io](https://img.shields.io/crates/v/crusty_traits_macros.svg)](https://crates.io/crates/crusty_traits_macros)
[![License](https://img.shields.io/crates/l/crusty_traits_macros.svg)](https://github.com/n1ght-hunter/crusty_traits/blob/master/LICENSE)
[![Downloads](https://img.shields.io/crates/d/crusty_traits_macros.svg)](https://crates.io/crates/crusty_traits_macros)

</div>

Procedural macros for crusty_traits - FFI-safe trait generation.

## Overview

This crate provides the `#[crusty_trait]` procedural macro that transforms Rust traits into FFI-safe equivalents with C-compatible vtables.

## The `crusty_trait` Macro

The main export of this crate is the `crusty_trait` attribute macro that:

- Generates C-compatible vtable structs
- Creates function pointers for trait methods
- Implements proper memory management
- Handles trait object lifecycle
- Supports super traits and complex inheritance

## Usage

This crate is re-exported by the main `crusty_traits` crate:

```toml
[dependencies]
crusty_traits = "0.1"
```

```rust
use crusty_traits::prelude::*;

#[crusty_trait]
pub trait MyTrait {
    fn method(&self, value: i32) -> i32;
}
```

For direct usage:

```toml
[dependencies]
crusty_traits_macros = "0.1"
```

## Documentation

See the [API documentation](https://docs.rs/crusty_traits_macros) for detailed usage information.
