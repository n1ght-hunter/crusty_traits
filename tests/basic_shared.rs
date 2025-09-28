#![allow(missing_docs, unsafe_code)]

use std::{ffi::OsStr, path::PathBuf};

fn top_level() -> PathBuf {
    let top_dir = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .expect("Failed to get top directory")
        .stdout;
    let top_dir = String::from_utf8_lossy(&top_dir);
    PathBuf::from(top_dir.trim())
}

fn c_dll_path() -> PathBuf {
    let top_dir = top_level();
    top_dir.join("target").join("c").join("basic_c.dll")
}

fn rust_dll_path() -> PathBuf {
    let top_dir = top_level();
    top_dir.join("target").join("release").join("shared_lib.dll")
}

struct BasicSharedLib {
    lib: libloading::Library,
}

impl BasicSharedLib {
    fn new(path: impl AsRef<OsStr>) -> Self {
        let lib = unsafe { libloading::Library::new(path) }.expect("Failed to load C library");
        Self { lib }
    }

    fn hello_world(&self) {
        unsafe {
            let hello: libloading::Symbol<unsafe extern "C" fn()> = self
                .lib
                .get(b"hello_world\0")
                .expect("Failed to load hello_world");
            hello();
        }
    }

    fn add(&self, a: isize, b: isize) -> isize {
        unsafe {
            let add: libloading::Symbol<unsafe extern "C" fn(isize, isize) -> isize> =
                self.lib.get(b"add\0").expect("Failed to load add");
            add(a, b)
        }
    }

    fn multiply(&self, a: isize, b: isize) -> isize {
        unsafe {
            let multiply: libloading::Symbol<unsafe extern "C" fn(isize, isize) -> isize> = self
                .lib
                .get(b"multiply\0")
                .expect("Failed to load multiply");
            multiply(a, b)
        }
    }
}

#[test]
fn basic_c() {
    let lib_path = c_dll_path();

    assert!(lib_path.exists(), "C library was not built successfully");

    let lib = BasicSharedLib::new(lib_path);

    lib.hello_world();

    assert_eq!(lib.add(2, 3), 5);
    assert_eq!(lib.multiply(2, 3), 6);
}

#[test]
fn basic_rust() {
    let lib_path = rust_dll_path();

    assert!(lib_path.exists(), "Rust library was not built successfully");

    let lib = BasicSharedLib::new(lib_path);

    lib.hello_world();

    assert_eq!(lib.add(2, 3), 5);
    assert_eq!(lib.multiply(2, 3), 6);
}
