//! This crate provides a means of creating C-compatible vtables for Rust traits.
//!

pub use crusty_traits_core::*;
pub use crusty_traits_macros::crusty_trait;

/// Modules that exports all the necessary types and traits for FFI.
pub mod prelude {
    pub use crate::AsVTable;
    pub use crate::CDrop;
    pub use crate::CRef;
    pub use crate::CRefMut;
    pub use crate::CRepr;
    pub use crate::crusty_trait;
}

pub use crusty_traits_types as types;

#[cfg(test)]
mod tests {
    use super::*;

    #[crusty_trait]
    trait Buffer: Send + Sync {
        fn as_slice(&self) -> *mut u8;
        fn extend(&mut self, amount: usize);
        fn len(&self) -> usize;
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
        let mut buffer = Vec::new();
        Buffer::extend(&mut buffer, 10);
        assert_eq!(buffer.len(), 10);
    }

    #[test]
    fn test_c_repr() {
        let buffer = Vec::new();
        let mut vtable = BufferVTable::new_boxed(buffer);
        vtable.extend(10);
        assert_eq!(vtable.len(), 10);
        let slice = vtable.as_slice();
        assert!(!slice.is_null());
        #[allow(unsafe_code)]
        {
            let slice = unsafe { std::slice::from_raw_parts_mut(slice as *mut _, vtable.len()) };
            assert_eq!(slice.len(), 10);
            for i in 0..10 {
                slice[i] = i as u8;
            }
        }
        let slice = vtable.as_slice();
        #[allow(unsafe_code)]
        {
            let slice = unsafe { std::slice::from_raw_parts(slice as *const _, vtable.len()) };
            assert_eq!(slice.len(), 10);
            for i in 0..10 {
                assert_eq!(slice[i], i as u8);
            }
        }
    }
}
