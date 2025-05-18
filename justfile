

# list all available recipes
default: 
    @just --list


test: cargo-test miri-test

cargo-test:
    cargo test --all-features --release --all

miri-test:
    cargo +nightly miri test --all-features --all