#![allow(missing_docs)]

use crusty_traits::prelude::*;

#[crusty_trait]
trait SuperBuffer {
    fn test(&self) -> usize;
}

#[crusty_trait]
trait Buffer: SuperBuffer + Send + Sync {
    fn as_slice(&self) -> *mut u8;
    fn extend(&mut self, amount: usize);
    fn capacity(&self) -> usize;
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
        self.reserve(amount);
    }

    fn capacity(&self) -> usize {
        self.capacity()
    }

    fn len(&self) -> usize {
        self.len()
    }
}

#[test]
fn super_trait() {
    let buffer: Vec<u8> = Vec::new();
    let mut buffer = BufferVTable::new_boxed(buffer);
    let test = buffer.test();
    buffer.extend(10);
    assert_eq!(test, 0);
    assert!(buffer.capacity() >= 10);
}
