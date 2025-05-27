//! ```cargo
//! [dependencies]
//! cbindgen = {git = "https://github.com/n1ght-hunter/cbindgen.git", rev = "b0c107b"}
//! ```

use std::path::Path;

use cbindgen::Config;

#[test]
fn main() {
    let top_dir = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .expect("Failed to get top directory")
        .stdout;
    let top_dir = String::from_utf8_lossy(&top_dir);
    let top_dir = top_dir.trim();

    let config = Config::from_root_or_default(Path::new(top_dir).join("tests/cbindgen.toml"));

    cbindgen::Builder::new()
        .with_config(config)
        .with_crate(top_dir)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(Path::new(top_dir).join("tests/include/cbindgen.h"));

}
