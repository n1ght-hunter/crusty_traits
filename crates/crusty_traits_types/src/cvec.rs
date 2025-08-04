//! C-compatible vector types and traits. that converts to the rust Vec type.
use crusty_traits_core::*;
use crusty_traits_macros::crusty_trait;

#[crusty_trait]
trait CVec<V> {
    fn push(&mut self, value: V);
    fn as_slice(&self) -> *mut V;
    fn extend(&mut self, amount: usize);
    fn capacity(&self) -> usize;
    fn len(&self) -> usize;
}

impl<T> CVec<T> for Vec<T> {
    fn push(&mut self, value: T) {
        self.push(value);
    }

    fn as_slice(&self) -> *mut T {
        self.as_ptr() as *mut T
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

#[cfg(test)]
mod tests {
    #![allow(unsafe_code)]
    use super::*;

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

        // Check if the slice is valid
        {
            let slice = cvec.as_slice();
            assert!(!slice.is_null());
        }
        cvec.push(TestData {
            string: "Hello".to_string(),
            number: 42,
        });

        {
            let slice = cvec.as_slice();
            let slice = unsafe { std::slice::from_raw_parts_mut(slice, cvec.len()) };

            println!("slice : {:?}", slice);

            // for i in 0..3 {
            //     slice[i] = TestData {
            //         string: "test".to_string(),
            //         number: 42,
            //     };
            // }
        }
    }
}
