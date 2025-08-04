

[private]
help:
    @just --list scripts

build_cbindgen:
    cargo run --bin build_cbindgen


build_c: build_cbindgen
    cargo run --bin build_c