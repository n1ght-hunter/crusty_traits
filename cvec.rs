pub mod cvec {
    //! C-compatible vector types and traits. that converts to the rust Vec type.
    //!
    use crusty_traits_core::*;
    use crusty_traits_macros::crusty_trait;
    use crate::cslice::{CSlice, CSliceVTable};
    trait CVec<V>: CSlice<V> {
        fn push(&mut self, value: V);
        fn extend(&mut self, amount: usize);
        fn capacity(&self) -> usize;
        fn len(&self) -> usize;
    }
    #[repr(C)]
    ///A repr C vtable for the trait CVec
    struct CVecVTable<V> {
        pub push: unsafe extern "C" fn(CRefMut<Self>, V),
        pub extend: unsafe extern "C" fn(CRefMut<Self>, usize),
        pub capacity: unsafe extern "C" fn(CRef<Self>) -> usize,
        pub len: unsafe extern "C" fn(CRef<Self>) -> usize,
        pub super_c_slice: &'static CSliceVTable<V>,
        /// A function pointer to the drop function for the trait
        pub drop: unsafe extern "C" fn(CRefMut<Self>),
    }
    impl AsVTable<&'static CSliceVTable<V>> for CVecVTable {
        fn as_vtable(&self) -> &'static CSliceVTable<V> {
            &self.super_c_slice
        }
    }
    impl<V> CDrop for CVecVTable<V> {
        fn drop(repr: CRefMut<Self>) {
            unsafe { (repr.get_vtable().drop)(repr) }
        }
    }
    impl<V: 'static> CVecVTable<V> {
        /// Creates a new vtable for the type GEN that implements the trait
        pub fn new_boxed<GEN: CVec<V> + 'static>(input: GEN) -> CRepr<CVecVTable<V>> {
            let vtable = CVecVTable::create_vtable::<GEN>();
            CRepr::new_boxed(vtable, input)
        }
        /// Creates a new vtable for the type GEN then store in a static variable in the heap
        pub fn create_vtable<GEN: CVec<V> + 'static>() -> &'static CVecVTable<V> {
            static FN_MAP: std::sync::LazyLock<
                std::sync::Mutex<
                    std::collections::HashMap<
                        std::any::TypeId,
                        &'static (dyn std::any::Any + Send + Sync),
                    >,
                >,
            > = std::sync::LazyLock::new(|| std::sync::Mutex::new(
                std::collections::HashMap::new(),
            ));
            let type_id = std::any::TypeId::of::<GEN>();
            let mut map = FN_MAP.lock().unwrap();
            let entry = map
                .entry(type_id)
                .or_insert_with(|| {
                    let vtable = Box::new(CVecVTable {
                        push: {
                            unsafe extern "C" fn push<VMETHOD, GEN: CVec<VMETHOD>>(
                                arg_0: CRefMut<CVecVTable<VMETHOD>>,
                                arg_1: VMETHOD,
                            ) {
                                #[allow(unsafe_code)]
                                unsafe {
                                    GEN::push(&mut *(arg_0.as_ptr() as *mut GEN), arg_1)
                                }
                            }
                            push::<V, GEN>
                        },
                        extend: {
                            unsafe extern "C" fn extend<VMETHOD, GEN: CVec<VMETHOD>>(
                                arg_0: CRefMut<CVecVTable<VMETHOD>>,
                                arg_1: usize,
                            ) {
                                #[allow(unsafe_code)]
                                unsafe {
                                    GEN::extend(&mut *(arg_0.as_ptr() as *mut GEN), arg_1)
                                }
                            }
                            extend::<V, GEN>
                        },
                        capacity: {
                            unsafe extern "C" fn capacity<VMETHOD, GEN: CVec<VMETHOD>>(
                                arg_0: CRef<CVecVTable<VMETHOD>>,
                            ) -> usize {
                                #[allow(unsafe_code)]
                                unsafe { GEN::capacity(&*(arg_0.as_ptr() as *const GEN)) }
                            }
                            capacity::<V, GEN>
                        },
                        len: {
                            unsafe extern "C" fn len<VMETHOD, GEN: CVec<VMETHOD>>(
                                arg_0: CRef<CVecVTable<VMETHOD>>,
                            ) -> usize {
                                #[allow(unsafe_code)]
                                unsafe { GEN::len(&*(arg_0.as_ptr() as *const GEN)) }
                            }
                            len::<V, GEN>
                        },
                        super_c_slice: CSliceVTable::create_vtable::<GEN>(),
                        drop: {
                            unsafe extern "C" fn drop<VMETHOD, GEN: CVec<VMETHOD>>(
                                arg_0: CRefMut<CVecVTable<VMETHOD>>,
                            ) {
                                #[allow(unsafe_code)]
                                unsafe {
                                    ::core::mem::drop(
                                        Box::from_raw(arg_0.as_ptr() as *mut GEN),
                                    );
                                }
                            }
                            drop::<V, GEN>
                        },
                    });
                    Box::leak(vtable)
                });
            entry.downcast_ref().unwrap()
        }
    }
    impl<V> CVec<V> for CRepr<CVecVTable<V>> {
        fn push(&mut self, value: V) {
            #[allow(unsafe_code)]
            unsafe { (self.get_vtable().push)(self.as_cref_mut(), value) }
        }
        fn extend(&mut self, amount: usize) {
            #[allow(unsafe_code)]
            unsafe { (self.get_vtable().extend)(self.as_cref_mut(), amount) }
        }
        fn capacity(&self) -> usize {
            #[allow(unsafe_code)] unsafe { (self.get_vtable().capacity)(self.as_cref()) }
        }
        fn len(&self) -> usize {
            #[allow(unsafe_code)] unsafe { (self.get_vtable().len)(self.as_cref()) }
        }
    }
    impl<GEN, V> CVec<V> for CRepr<GEN>
    where
        GEN: AsVTable<&'static CVecVTable<V>> + CDrop + AsVTable<&'static CSliceVTable>,
        V: 'static,
    {
        fn push(&mut self, value: V) {
            let methods: &'static CVecVTable<V> = self.as_vtable();
            #[allow(unsafe_code)]
            unsafe {
                (methods
                    .push)(
                    self.as_cref_mut_with_methods(std::ptr::NonNull::from(methods)),
                    value,
                )
            }
        }
        fn extend(&mut self, amount: usize) {
            let methods: &'static CVecVTable<V> = self.as_vtable();
            #[allow(unsafe_code)]
            unsafe {
                (methods
                    .extend)(
                    self.as_cref_mut_with_methods(std::ptr::NonNull::from(methods)),
                    amount,
                )
            }
        }
        fn capacity(&self) -> usize {
            let methods: &'static CVecVTable<V> = self.as_vtable();
            #[allow(unsafe_code)]
            unsafe {
                (methods
                    .capacity)(
                    self.as_cref_with_methods(std::ptr::NonNull::from(methods)),
                )
            }
        }
        fn len(&self) -> usize {
            let methods: &'static CVecVTable<V> = self.as_vtable();
            #[allow(unsafe_code)]
            unsafe {
                (methods
                    .len)(self.as_cref_with_methods(std::ptr::NonNull::from(methods)))
            }
        }
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
        fn len(&self) -> usize {
            self.len()
        }
    }
}
