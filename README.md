# Crusty Traits 
## C <-> Rust Traits

A crate that creates a macro and supporting code to allow for traits to be FFI-safe using C ABI.
> [!WARNING]
> This crate uses unsafe code and may be unsound if used incorrectly. Use at your own risk.
> If any issues are found please open an issue or a PR.

## Usage
Add the following to your Cargo.toml

```toml
[dependencies]  
crusty_traits = "0.1"
```

Then in your code
```rust
use crusty_traits::prelude::*;

#[crusty_trait]
pub trait MyTrait {
    fn method1(&self);
    fn method2(&mut self, value: i32) -> i32;
}
```

<details>

<summary>Roughly expands to the following</summary>

```rust
use crusty_traits::prelude::*;
pub trait MyTrait {
    fn method1(&self);
    fn method2(&mut self, value: i32) -> i32;
}
#[repr(C)]
///A repr C vtable for the trait MyTrait
pub struct MyTraitVTable {
    pub method1: unsafe extern "C" fn(CRef<MyTraitVTable>),
    pub method2: unsafe extern "C" fn(CRefMut<MyTraitVTable>, i32) -> i32,
    ///A function pointer to the drop function for the trait
    pub drop: unsafe extern "C" fn(CRefMut<MyTraitVTable>),
}
impl CDrop for MyTraitVTable {
    fn drop(repr: CRefMut<Self>) {
        unsafe { (repr.get_vtable().drop)(repr) }
    }
}
impl MyTraitVTable {
    /// Creates a new vtable for the type GEN that implements the trait
    pub fn new_boxed<GEN: MyTrait + 'static>(input: GEN) -> CRepr<MyTraitVTable> {
        let vtable = MyTraitVTable::create_vtable::<GEN>();
        CRepr::new_boxed(vtable, input)
    }
    /// Creates a new vtable for the type GEN then store in a static variable in the heap
    pub fn create_vtable<GEN: MyTrait + 'static>() -> &'static MyTraitVTable {
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
                let vtable = Box::new(MyTraitVTable {
                    method1: {
                        unsafe extern "C" fn method1<GEN: MyTrait>(
                            arg0: CRef<MyTraitVTable>,
                        ) {
                            #[allow(unsafe_code)]
                            unsafe { GEN::method1(&*(arg0.as_ptr() as *const GEN)) }
                        }
                        method1::<GEN>
                    },
                    method2: {
                        unsafe extern "C" fn method2<GEN: MyTrait>(
                            arg0: CRefMut<MyTraitVTable>,
                            arg1: i32,
                        ) -> i32 {
                            #[allow(unsafe_code)]
                            unsafe {
                                GEN::method2(&mut *(arg0.as_ptr() as *mut GEN), arg1)
                            }
                        }
                        method2::<GEN>
                    },
                    drop: {
                        unsafe extern "C" fn drop<GEN: MyTrait>(
                            arg_0: CRefMut<MyTraitVTable>,
                        ) {
                            #[allow(unsafe_code)]
                            unsafe {
                                ::core::mem::drop(
                                    Box::from_raw(arg_0.as_ptr() as *mut GEN),
                                );
                            }
                        }
                        drop::<GEN>
                    },
                });
                Box::leak(vtable)
            });
        entry.downcast_ref().unwrap()
    }
}
impl MyTrait for CRepr<MyTraitVTable> {
    fn method1(&self) {
        #[allow(unsafe_code)] 
        unsafe { (self.get_vtable().method1)(self.as_cref()) }
    }
    fn method2(&mut self, value: i32) -> i32 {
        #[allow(unsafe_code)]
        unsafe { (self.get_vtable().method2)(self.as_cref_mut(), value) }
    }
}
impl<GEN> MyTrait for CRepr<GEN>
where
    GEN: AsVTable<&'static MyTraitVTable> + CDrop,
{
    fn method1(&self) {
        let methods: &'static MyTraitVTable = self.as_vtable();
        #[allow(unsafe_code)]
        unsafe {
            (methods
                .method1)(
                self.as_cref_with_methods(std::ptr::NonNull::from(methods)),
            )
        }
    }
    fn method2(&mut self, value: i32) -> i32 {
        let methods: &'static MyTraitVTable = self.as_vtable();
        #[allow(unsafe_code)]
        unsafe {
            (methods
                .method2)(
                self.as_cref_mut_with_methods(std::ptr::NonNull::from(methods)),
                value,
            )
        }
    }
}

```
</details>

## Crate Details
This crate provides a macro `crusty_trait` that generates the necessary boilerplate code to create a C-compatible vtable for a given Rust trait.
This allows Rust traits to be used across FFI boundaries, making it easier to use Rust shared libraries or plugins in C or other languages that can interface with C.
Each trait that is annotated with `crusty_trait` will have a corresponding vtable struct generated, along with implementations for `CRepr` and `CDrop` to manage the memory and lifecycle of the trait objects.
The generated vtable struct will contain function pointers for each method in the trait, as well as a drop function to properly clean up the trait object when it is no longer needed. 
The trait is also implemented for `CRepr<MyTraitVTable>` and any `CRepr<GEN>` where `GEN` implements `AsVTable<&'static MyTraitVTable>`(used for super/sub traits) and `CDrop`, allowing for seamless usage of the trait across FFI boundaries in Rust code.