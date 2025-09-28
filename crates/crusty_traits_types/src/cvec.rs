//! C-compatible vector types and traits. that converts to the rust Vec type.
//!
use crusty_traits_core::*;
use crusty_traits_macros::crusty_trait;

use crate::cslice::{CSlice, CSliceVTable};

#[crusty_trait]
/// A trait that represents a C-compatible vector.
pub trait CVec<V>: CSlice<V> {
    /// Adds an element to the end of the vector.
    fn push(&mut self, value: V);
    /// Extends the vector's capacity by the given amount.
    fn extend(&mut self, amount: usize);
    /// Returns the capacity of the vector.
    fn capacity(&self) -> usize;
}

impl<T> CVec<T> for Vec<T> {
    fn push(&mut self, value: T) {
        self.push(value);
    }

    fn extend(&mut self, amount: usize) {
        self.reserve(amount);
    }

    fn capacity(&self) -> usize {
        self.capacity()
    }
}

#[cfg(test)]
mod tests {
    #![allow(unsafe_code)]
    use super::*;
    use crate::cslice::CSliceExt;

    #[repr(C)]
    #[derive(Debug, Clone)]
    struct TestData {
        string: String,
        number: usize,
    }

    #[test]
    fn test_cvec() {
        let vec: Vec<TestData> = Vec::new();
        let cvec = CVecVTable::new_boxed(vec);
        test_cvec_inner(cvec);
    }

    fn test_cvec_inner(mut cvec: impl CVec<TestData>) {
        cvec.extend(10);
        assert_eq!(cvec.len(), 0);
        assert!(cvec.capacity() >= 10);

        cvec.push(TestData {
            string: "Hello".to_string(),
            number: 42,
        });

        {
            let slice = cvec.as_slice();

            assert!(slice.len() > 0);
            assert_eq!(slice[0].string, "Hello");
            assert_eq!(slice[0].number, 42);
        }
    }
}
