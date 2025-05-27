#![feature(prelude_import)]
#![allow(missing_docs)]
#[prelude_import]
use std::prelude::rust_2024::*;
#[macro_use]
extern crate std;
use crusty_traits::prelude::*;
trait SuperBuffer {
    fn test(&self) -> usize;
}
#[repr(C)]
///A repr C vtable for the trait SuperBuffer
struct SuperBufferVTable {
    pub test: unsafe extern "C" fn(CRef<Self>) -> usize,
    /// A function pointer to the drop function for the trait
    pub drop: unsafe extern "C" fn(CRefMut<Self>),
}
impl<T> SuperBuffer for CRepr<T>
where
    T: AsVTable<&'static SuperBufferVTable> + CDrop,
{
    fn test(&self) -> usize {
        let methods = self.as_vtable();
        #[allow(unsafe_code)]
        unsafe {
            (methods.test)(self.as_cref_with_methods(std::ptr::NonNull::from(methods)))
        }
    }
}
impl CDrop for SuperBufferVTable {
    fn drop(repr: CRefMut<Self>) {
        unsafe { (repr.get_vtable().drop)(repr) }
    }
}
impl SuperBufferVTable {
    /// Creates a new vtable for the type T that implements the trait
    pub fn new_boxed<T: SuperBuffer + 'static>(input: T) -> CRepr<SuperBufferVTable> {
        let vtable = SuperBufferVTable::create_vtable::<T>();
        CRepr::new_boxed(vtable, input)
    }
    /// Creates a new vtable for the type T then store in a static variable in a hea
    pub fn create_vtable<T: SuperBuffer + 'static>() -> &'static SuperBufferVTable {
        static FN_MAP: std::sync::LazyLock<
            std::sync::Mutex<
                std::collections::HashMap<std::any::TypeId, &'static SuperBufferVTable>,
            >,
        > = std::sync::LazyLock::new(|| std::sync::Mutex::new(
            std::collections::HashMap::new(),
        ));
        let type_id = std::any::TypeId::of::<T>();
        let mut map = FN_MAP.lock().unwrap();
        let entry = map
            .entry(type_id)
            .or_insert_with(|| {
                let vtable = Box::new(SuperBufferVTable {
                    test: {
                        unsafe extern "C" fn test<T: SuperBuffer>(
                            arg_0: CRef<SuperBufferVTable>,
                        ) -> usize {
                            #[allow(unsafe_code)]
                            unsafe { T::test(&*(arg_0.as_ptr() as *const T)) }
                        }
                        test::<T>
                    },
                    drop: {
                        unsafe extern "C" fn drop<T: SuperBuffer>(
                            arg_0: CRefMut<SuperBufferVTable>,
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
impl SuperBuffer for CRepr<SuperBufferVTable> {
    fn test(&self) -> usize {
        #[allow(unsafe_code)] unsafe { (self.get_vtable().test)(self.as_cref()) }
    }
}
impl<T> SuperBuffer for CRepr<T>
where
    T: AsVTable<&'static SuperBufferVTable> + CDrop,
{
    fn test(&self) -> usize {
        let methods = self.as_vtable();
        #[allow(unsafe_code)]
        unsafe {
            (methods.test)(self.as_cref_with_methods(std::ptr::NonNull::from(methods)))
        }
    }
}
trait Buffer: SuperBuffer {
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
    pub super_super_buffer: &'static SuperBufferVTable,
    /// A function pointer to the drop function for the trait
    pub drop: unsafe extern "C" fn(CRefMut<Self>),
}
impl AsVTable<&'static SuperBufferVTable> for BufferVTable {
    fn as_vtable(&self) -> &'static SuperBufferVTable {
        &self.super_super_buffer
    }
}
impl<T> Buffer for CRepr<T>
where
    T: AsVTable<&'static BufferVTable> + CDrop,
{
    fn as_slice(&self) -> *mut u8 {
        let methods = self.as_vtable();
        #[allow(unsafe_code)]
        unsafe {
            (methods
                .as_slice)(self.as_cref_with_methods(std::ptr::NonNull::from(methods)))
        }
    }
    fn extend(&mut self, amount: usize) {
        let methods = self.as_vtable();
        #[allow(unsafe_code)]
        unsafe {
            (methods
                .extend)(
                self.as_cref_mut_with_methods(std::ptr::NonNull::from(methods)),
                amount,
            )
        }
    }
    fn len(&self) -> usize {
        let methods = self.as_vtable();
        #[allow(unsafe_code)]
        unsafe {
            (methods.len)(self.as_cref_with_methods(std::ptr::NonNull::from(methods)))
        }
    }
}
impl CDrop for BufferVTable {
    fn drop(repr: CRefMut<Self>) {
        unsafe { (repr.get_vtable().drop)(repr) }
    }
}
impl BufferVTable {
    /// Creates a new vtable for the type T that implements the trait
    pub fn new_boxed<T: Buffer + 'static>(input: T) -> CRepr<BufferVTable> {
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
                            unsafe { T::extend(&mut *(arg_0.as_ptr() as *mut T), arg_1) }
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
                    super_super_buffer: SuperBufferVTable::create_vtable::<T>(),
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
        self.extend_from_slice(&::alloc::vec::from_elem(0, amount));
    }
    fn len(&self) -> usize {
        self.len()
    }
}
extern crate test;
#[rustc_test_marker = "test_crusty_trait"]
#[doc(hidden)]
pub const test_crusty_trait: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("test_crusty_trait"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests\\super_trait.rs",
        start_line: 51usize,
        start_col: 4usize,
        end_line: 51usize,
        end_col: 21usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(test_crusty_trait()),
    ),
};
fn test_crusty_trait() {
    let buffer: Vec<u8> = Vec::new();
    let mut buffer = BufferVTable::new_boxed(buffer);
    let test = buffer.test();
    buffer.extend(10);
    match (&test, &0) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    match (&buffer.len(), &10) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
}
#[rustc_main]
#[coverage(off)]
#[doc(hidden)]
pub fn main() -> () {
    extern crate test;
    test::test_main_static(&[&test_crusty_trait])
}
