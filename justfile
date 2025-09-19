
mod scripts "scripts/scripts.just"

# list all available recipes
default: 
    @just --list


prepare:


test: cargo-test miri-test

cargo-test:
    cargo test --all-features --release --all

miri-test:
    cargo +nightly miri test --all-features --all


