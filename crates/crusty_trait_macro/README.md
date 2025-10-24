# crusty_trait_macro

<div align="center">

[![Documentation](https://docs.rs/crusty_trait_macro/badge.svg)](https://docs.rs/crusty_trait_macro)
[![Crates.io](https://img.shields.io/crates/v/crusty_trait_macro.svg)](https://crates.io/crates/crusty_trait_macro)
[![License](https://img.shields.io/crates/l/crusty_trait_macro.svg)](https://github.com/n1ght-hunter/crusty_traits/blob/master/LICENSE)
[![Downloads](https://img.shields.io/crates/d/crusty_trait_macro.svg)](https://crates.io/crates/crusty_trait_macro)

</div>

Internal procedural macro implementation for crusty_traits.

## Overview

This crate contains the internal implementation details of the procedural macros used by `crusty_traits`. It handles the code generation for creating FFI-safe vtables from Rust traits.

## Internal Implementation

This crate is an implementation detail and is not intended for direct use. It provides:

- AST parsing and manipulation for trait definitions
- Vtable struct generation
- C-compatible function pointer creation
- Memory management code generation
- Super trait handling

## Usage

This crate is used internally by `crusty_traits_macros` and should not be used directly. Use the main `crusty_traits` crate instead:

```toml
[dependencies]
crusty_traits = "0.1"
```

## Documentation

See the [API documentation](https://docs.rs/crusty_trait_macro) for implementation details.
