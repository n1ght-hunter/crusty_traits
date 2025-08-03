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

impl<T: ?Sized> Inner<T> {
    #[allow(unsafe_code)]
    /// Creates a new `Inner` from a vtable and context.
    pub unsafe fn map_vtable<U: ?Sized, F: FnOnce(&T) -> NonNull<U>>(self, map: F) -> Inner<U> {
        Inner {
            vtable: map(unsafe { self.vtable.as_ref() }),
            ptr: self.ptr,
        }
    }
}

impl<T: ?Sized> Copy for Inner<T> {}
impl<T: ?Sized> Clone for Inner<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Deref for Inner<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        #[allow(unsafe_code)]
        unsafe {
            self.vtable.as_ref()
        }
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

impl<T: CDrop> Deref for CRepr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        #[allow(unsafe_code)]
        unsafe {
            self.inner.vtable.as_ref()
        }
    }
}

impl<T: CDrop> CRepr<T> {
    /// Creates a new `CRepr` from a vtable and context.
    pub fn new_boxed<C>(vtable: &'static T, context: C) -> Self {
        let context = Box::new(context);
        let vtable = NonNull::from(vtable);
        let context = NonNull::from(Box::leak(context)).cast();

        #[allow(unsafe_code)]
        // SAFETY: The vtable and context are valid and properly aligned.
        unsafe {
            Self::from_raw_parts(vtable, context)
        }
    }

    /// Creates a new `CRepr` from a vtable and context.
    /// # Safety
    /// The caller must ensure that the vtable and context are valid and properly aligned.
    #[allow(unsafe_code)]
    pub unsafe fn from_raw_parts(vtable: NonNull<T>, context: NonNull<u8>) -> Self {
        Self {
            inner: Inner {
                vtable,
                ptr: context,
            },
        }
    }
}

impl<T: CDrop + ?Sized> CRepr<T> {
    /// Maps the vtable to a new type using the provided function.
    #[allow(unsafe_code)]
    pub unsafe fn as_cref_with_methods<U: ?Sized>(&self, methods: NonNull<U>) -> CRef<U> {
        CRef {
            inner: unsafe { self.inner.map_vtable(|_| methods) },
            phantom: std::marker::PhantomData,
        }
    }

    /// Maps the vtable to a new type using the provided function.
    #[allow(unsafe_code)]
    pub unsafe fn as_cref_mut_with_methods<U: ?Sized>(
        &mut self,
        methods: NonNull<U>,
    ) -> CRefMut<U> {
        CRefMut {
            inner: unsafe { self.inner.map_vtable(|_| methods) },
            phantom: std::marker::PhantomData,
        }
    }

    /// Returns a pointer to the context.
    pub fn as_ptr(&self) -> *const u8 {
        self.inner.ptr.as_ptr()
    }

    /// Returns a reference to the vtable.
    pub fn get_vtable(&self) -> &T {
        #[allow(unsafe_code)]
        unsafe {
            self.inner.vtable.as_ref()
        }
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
    /// Creates a new `CRef` from a vtable and context.
    #[allow(unsafe_code)]
    pub unsafe fn from_raw_parts(vtable: NonNull<T>, context: NonNull<u8>) -> Self {
        Self {
            inner: Inner {
                vtable,
                ptr: context,
            },
            phantom: std::marker::PhantomData,
        }
    }

    /// Returns a pointer to the context.
    pub fn as_ptr(&self) -> *const u8 {
        self.inner.ptr.as_ptr()
    }

    /// Returns a reference to the vtable.
    pub fn get_vtable(&self) -> &T {
        #[allow(unsafe_code)]
        unsafe {
            self.inner.vtable.as_ref()
        }
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
        #[allow(unsafe_code)]
        unsafe {
            self.inner.vtable.as_ref()
        }
    }
}

impl<T: ?Sized> DerefMut for CRef<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        #[allow(unsafe_code)]
        unsafe {
            self.inner.vtable.as_mut()
        }
    }
}

macro_rules! impl_c_ref {
    ($name:ident) => {
        impl<T: ?Sized> Copy for $name<'_, T> {}
        impl<T: ?Sized> Clone for $name<'_, T> {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl<T: ?Sized> Deref for $name<'_, T> {
            type Target = T;
            fn deref(&self) -> &Self::Target {
                #[allow(unsafe_code)]
                unsafe {
                    self.inner.vtable.as_ref()
                }
            }
        }
    };
}

impl_c_ref!(CRef);
impl_c_ref!(CRefMut);
