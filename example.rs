#![feature(prelude_import)]
//! This crate provides a means of creating C-compatible vtables for Rust traits.
//!
#[prelude_import]
use std::prelude::rust_2024::*;
#[macro_use]
extern crate std;
mod trait_wrapper {
    use std::{
        ops::{Deref, DerefMut},
        ptr::NonNull,
    };
    use crate::CDrop;
    #[repr(C)]
    struct Inner<T: ?Sized> {
        pub vtable: NonNull<T>,
        pub ptr: NonNull<u8>,
    }
    impl<T: ?Sized> Copy for Inner<T> {}
    impl<T: ?Sized> Clone for Inner<T> {
        fn clone(&self) -> Self {
            *self
        }
    }
    /// A trait that represents a buffer that can be converted to a C-compatible slice.
    #[repr(transparent)]
    pub struct CRepr<T: CDrop + ?Sized> {
        inner: Inner<T>,
    }
    #[allow(unsafe_code)]
    unsafe impl<T: Send + CDrop + ?Sized> Send for CRepr<T> {}
    #[allow(unsafe_code)]
    unsafe impl<T: Sync + CDrop + ?Sized> Sync for CRepr<T> {}
    impl<T: CDrop> CRepr<T> {
        /// Creates a new `CRepr` from a vtable and context.
        pub fn new_boxed<C>(vtable: &'static T, context: C) -> Self {
            let context = Box::new(context);
            let vtable = NonNull::from(vtable);
            let context = NonNull::from(Box::leak(context)).cast();
            Self {
                inner: Inner { vtable, ptr: context },
            }
        }
    }
    impl<T: CDrop + ?Sized> CRepr<T> {
        /// Returns a pointer to the context.
        pub fn as_ptr(&self) -> *const u8 {
            self.inner.ptr.as_ptr()
        }
        /// Returns a reference to the vtable.
        pub fn get_vtable(&self) -> &T {
            #[allow(unsafe_code)] unsafe { self.inner.vtable.as_ref() }
        }
        /// Returns a cref
        pub fn as_cref(&self) -> CRef<T> {
            CRef {
                inner: self.inner,
                phantom: std::marker::PhantomData,
            }
        }
        /// Returns a cref mut
        pub fn as_cref_mut(&mut self) -> CRefMut<T> {
            CRefMut {
                inner: self.inner,
                phantom: std::marker::PhantomData,
            }
        }
    }
    impl<T: CDrop + ?Sized> Drop for CRepr<T> {
        fn drop(&mut self) {
            T::drop(self.as_cref_mut());
        }
    }
    #[repr(transparent)]
    /// A reference to a C-compatible object.
    pub struct CRef<'a, T: ?Sized> {
        inner: Inner<T>,
        phantom: std::marker::PhantomData<&'a T>,
    }
    impl<'a, T: ?Sized> CRef<'a, T> {
        /// Returns a pointer to the context.
        pub fn as_ptr(&self) -> *const u8 {
            self.inner.ptr.as_ptr()
        }
        /// Returns a reference to the vtable.
        pub fn get_vtable(&self) -> &T {
            #[allow(unsafe_code)] unsafe { self.inner.vtable.as_ref() }
        }
    }
    #[repr(transparent)]
    /// A reference to a C-compatible object.
    pub struct CRefMut<'a, T: ?Sized> {
        inner: Inner<T>,
        phantom: std::marker::PhantomData<&'a mut T>,
    }
    impl<'a, T: ?Sized> CRefMut<'a, T> {
        /// Returns a pointer to the context.
        pub fn as_ptr(&self) -> *const u8 {
            self.inner.ptr.as_ptr()
        }
        /// Returns a reference to the vtable.
        pub fn get_vtable(&self) -> &T {
            #[allow(unsafe_code)] unsafe { self.inner.vtable.as_ref() }
        }
    }
    impl<T: ?Sized> DerefMut for CRef<'_, T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            #[allow(unsafe_code)] unsafe { self.inner.vtable.as_mut() }
        }
    }
    impl<T: ?Sized> Copy for CRef<'_, T> {}
    impl<T: ?Sized> Clone for CRef<'_, T> {
        fn clone(&self) -> Self {
            *self
        }
    }
    impl<T: ?Sized> Deref for CRef<'_, T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            #[allow(unsafe_code)] unsafe { self.inner.vtable.as_ref() }
        }
    }
    impl<T: ?Sized> Copy for CRefMut<'_, T> {}
    impl<T: ?Sized> Clone for CRefMut<'_, T> {
        fn clone(&self) -> Self {
            *self
        }
    }
    impl<T: ?Sized> Deref for CRefMut<'_, T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            #[allow(unsafe_code)] unsafe { self.inner.vtable.as_ref() }
        }
    }
}
pub use trait_wrapper::*;
pub use crusty_traits_macros::crusty_trait;
/// Modules that exports all the necessary types and traits for FFI.
pub mod prelude {
    pub use crate::crusty_trait;
    pub use crate::CRepr;
    pub use crate::CRef;
    pub use crate::CRefMut;
    pub use crate::CDrop;
}
/// A trait that represents dropping a Rust object in a C-compatible way.
pub trait CDrop {
    /// Drops the object represented by the given `CRepr`.
    fn drop(repr: CRefMut<Self>);
}
mod tests {
    use super::*;
    trait Buffer: Send + Sync {
        fn as_slice(&self) -> *mut u8;
        fn extend(&mut self, amount: usize);
        fn len(&self) -> usize;
    }
    #[repr(C)]
    ///A repr C vtable for the trait Buffer
    struct BufferVTable {
        pub as_slice: unsafe extern "C" fn(CRef<Self>) -> *mut u8,
        pub extend: unsafe extern "C" fn(CRefMut<Self>, usize),
        pub len: unsafe extern "C" fn(CRef<Self>) -> usize,
        /// A function pointer to the drop function for the trait
        pub drop: unsafe extern "C" fn(CRefMut<Self>),
    }
    impl CDrop for BufferVTable {
        fn drop(repr: CRefMut<Self>) {
            unsafe { (repr.get_vtable().drop)(repr) }
        }
    }
    impl BufferVTable {
        /// Creates a new vtable for the type T that implements the trait
        pub fn new_boxed<T: Buffer>(input: T) -> CRepr<BufferVTable> {
            let vtable = BufferVTable::create_vtable::<T>();
            CRepr::new_boxed(vtable, input)
        }
        /// Creates a new vtable for the type T then store in a static variable in a hea
        pub fn create_vtable<T: Buffer + 'static>() -> &'static BufferVTable {
            static FN_MAP: std::sync::LazyLock<
                std::sync::Mutex<
                    std::collections::HashMap<std::any::TypeId, &'static BufferVTable>,
                >,
            > = std::sync::LazyLock::new(|| std::sync::Mutex::new(
                std::collections::HashMap::new(),
            ));
            let type_id = std::any::TypeId::of::<T>();
            let mut map = FN_MAP.lock().unwrap();
            let entry = map
                .entry(type_id)
                .or_insert_with(|| {
                    let vtable = Box::new(BufferVTable {
                        as_slice: {
                            unsafe extern "C" fn as_slice<T: Buffer>(
                                arg_0: CRef<BufferVTable>,
                            ) -> *mut u8 {
                                #[allow(unsafe_code)]
                                unsafe { T::as_slice(&*(arg_0.as_ptr() as *const T)) }
                            }
                            as_slice::<T>
                        },
                        extend: {
                            unsafe extern "C" fn extend<T: Buffer>(
                                arg_0: CRefMut<BufferVTable>,
                                arg_1: usize,
                            ) {
                                #[allow(unsafe_code)]
                                unsafe {
                                    T::extend(&mut *(arg_0.as_ptr() as *mut T), arg_1)
                                }
                            }
                            extend::<T>
                        },
                        len: {
                            unsafe extern "C" fn len<T: Buffer>(
                                arg_0: CRef<BufferVTable>,
                            ) -> usize {
                                #[allow(unsafe_code)]
                                unsafe { T::len(&*(arg_0.as_ptr() as *const T)) }
                            }
                            len::<T>
                        },
                        drop: {
                            unsafe extern "C" fn drop<T: Buffer>(
                                arg_0: CRefMut<BufferVTable>,
                            ) {
                                #[allow(unsafe_code)]
                                unsafe {
                                    ::core::mem::drop(Box::from_raw(arg_0.as_ptr() as *mut T));
                                }
                            }
                            drop::<T>
                        },
                    });
                    Box::leak(vtable)
                });
            &entry
        }
    }
    impl Buffer for CRepr<BufferVTable> {
        fn as_slice(&self) -> *mut u8 {
            #[allow(unsafe_code)] unsafe { (self.get_vtable().as_slice)(self.as_cref()) }
        }
        fn extend(&mut self, amount: usize) {
            #[allow(unsafe_code)]
            unsafe { (self.get_vtable().extend)(self.as_cref_mut(), amount) }
        }
        fn len(&self) -> usize {
            #[allow(unsafe_code)] unsafe { (self.get_vtable().len)(self.as_cref()) }
        }
    }
    impl Buffer for Vec<u8> {
        fn as_slice(&self) -> *mut u8 {
            self.as_ptr() as *mut u8
        }
        fn extend(&mut self, amount: usize) {
            self.extend_from_slice(&::alloc::vec::from_elem(0, amount));
        }
        fn len(&self) -> usize {
            self.len()
        }
    }
}
