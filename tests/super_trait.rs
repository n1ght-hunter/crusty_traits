#![allow(missing_docs)]

use crusty_traits::prelude::*;

#[crusty_trait]
trait SuperBuffer {
    fn test(&self) -> usize;
}

#[crusty_trait]
trait Buffer: SuperBuffer {
    fn as_slice(&self) -> *mut u8;
    fn extend(&mut self, amount: usize);
    fn len(&self) -> usize;
}

impl SuperBuffer for Vec<u8> {
    fn test(&self) -> usize {
        self.len()
    }
}

impl Buffer for Vec<u8> {
    fn as_slice(&self) -> *mut u8 {
        self.as_ptr() as *mut u8
    }

    fn extend(&mut self, amount: usize) {
        self.extend_from_slice(&vec![0; amount]);
    }

    fn len(&self) -> usize {
        self.len()
    }
}

#[test]
fn test_crusty_trait() {
    let buffer: Vec<u8> = Vec::new();
    let mut buffer = BufferVTable::new_boxed(buffer);
    let test = buffer.test();
    buffer.extend(10);
    assert_eq!(test, 0);
    assert_eq!(buffer.len(), 10);
}

enum StringNum {
    String1(String),
    String2(String),
    String3(String),
    Static1,
    Static2,
    Static3,
}

fn extract_event_type(message: &StringNum) -> &str {
    match message {
        StringNum::String1(s) => s,
        StringNum::String2(s) => s,
        StringNum::String3(s) => s,
        StringNum::Static1 => "Static1",
        StringNum::Static2 => "Static2",
        StringNum::Static3 => "Static3"
    }
}
