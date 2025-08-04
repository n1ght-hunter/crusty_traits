use std::path::PathBuf;

use libloading::library_filename;

fn main() {
    let top_dir = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .expect("Failed to get top directory")
        .stdout;
    let top_dir = String::from_utf8_lossy(&top_dir);
    let top_dir = PathBuf::from(top_dir.trim());

    let out_dir = top_dir.join("target").join("c");

    let mut builder = cc::Build::new();

    #[cfg(windows)]
    {
        builder.compiler("clang");
    }

    builder
        .file(top_dir.join("tests/c/src/basic_c.c"))
        .target("x86_64-pc-windows-msvc")
        .opt_level(3)
        .out_dir(&out_dir)
        .host("x86_64-pc-windows-msvc");

    let objects = builder.compile_intermediates();

    builder
        .get_compiler()
        .to_command()
        .args(["-shared", "-o"])
        .arg(out_dir.join(library_filename("basic_c")))
        .args(&objects)
        .status()
        .expect("Failed to compile C library");
}
