use std::path::PathBuf;

use cbindgen::Config;

fn main() {
    let top_dir = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .expect("Failed to get top directory")
        .stdout;
    let top_dir = String::from_utf8_lossy(&top_dir);
    let top_dir = PathBuf::from(top_dir.trim());

    let config = Config::from_file(top_dir.join("tests/cbindgen.toml"))
        .expect("Failed to load cbindgen config");

    cbindgen::Builder::new()
        .with_crate_and_name(top_dir.join("testing_crates/shared_lib"), "shared_lib")
        .with_config(config)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(top_dir.join("tests/c/include/cbindgen.h"));
}
