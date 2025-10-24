//! macro to impl the crusty traits.

use proc_macro2::TokenStream;
use super_trait::get_super_traits;
use vtable::create_vtable;

mod cdrop;
mod super_trait;
mod trait_for_cref;
mod utils;
mod vtable;

pub(crate) const IGNORE_SUPER_TRAITS: [&str; 3] = ["Send", "Sync", "Sized"];

fn error_file(msg: TokenStream) -> syn::File {
    syn::File {
        shebang: None,
        attrs: vec![],
        items: vec![syn::Item::Verbatim(msg)],
    }
}

/// Generate the crusty trait and its vtable.
pub fn impl_crusty_trait(input: syn::ItemTrait) -> syn::File {
    let mut output = syn::File {
        shebang: None,
        attrs: vec![],
        items: vec![],
    };

    let super_traits = match get_super_traits(&input) {
        Ok(s) => s,
        Err(e) => return error_file(e.to_compile_error()),
    };
    let vtable = match create_vtable(&input, &super_traits.super_traits) {
        Ok(v) => v,
        Err(e) => return error_file(e.to_compile_error()),
    };
    let as_vtable_impls =
        super_trait::impl_as_vtable_for_super_traits(&super_traits.super_traits, &vtable)
            .collect::<Vec<_>>();

    let cdrop_impl = cdrop::impl_cdrop_for_vtable(&vtable);
    let vtable_methods = vtable::methods::impl_vtable_methods(&input, &vtable);
    let trait_for_cref = trait_for_cref::impl_trait_for_c_ref(&input, &vtable);
    let trait_for_cref_where_as_vtable = trait_for_cref::impl_trait_for_c_ref_where_as_vtable(
        &input,
        &vtable,
        &super_traits.super_traits,
        &super_traits.ignore_bounds,
    );

    output.items.push(input.into());
    output.items.push(vtable.into());
    output.items.extend(as_vtable_impls);
    output.items.push(cdrop_impl);
    output.items.push(vtable_methods);
    output.items.push(syn::Item::Impl(trait_for_cref));
    output
        .items
        .push(syn::Item::Impl(trait_for_cref_where_as_vtable));

    output
}

#[cfg(test)]
mod tests {
    use crate::utils::test_utils::item_to_pretty_string;

    use super::*;
    use syn::parse_quote;

    #[test]
    fn basic_test() {
        let input: syn::ItemTrait = parse_quote! {
            pub trait MyTrait {
                fn my_method(&self, x: i32) -> i32;
                fn another_method(&mut self, y: String);
            }
        };
        let output = impl_crusty_trait(input.clone());
        assert_eq!(output.items[0], syn::Item::Trait(input));

        let expected_vtable: syn::ItemStruct = parse_quote! {
            #[repr(C)]
            #[doc = "A repr C vtable for the trait MyTrait"]
            pub struct MyTraitVTable {
                pub my_method: unsafe extern "C" fn(CRef<MyTraitVTable>, i32) -> i32,
                pub another_method: unsafe extern "C" fn(CRefMut<MyTraitVTable>, String),
                #[doc = "A function pointer to the drop function for the trait"]
                pub drop: unsafe extern "C" fn(CRefMut<MyTraitVTable>),
            }
        };

        // Compare using pretty-printed representation to avoid structural metadata issues
        let generated_vtable = item_to_pretty_string(output.items[1].clone());
        let expected_vtable = item_to_pretty_string(syn::Item::Struct(expected_vtable.clone()));

        assert_eq!(
            generated_vtable, expected_vtable,
            "Generated vtable does not match expected vtable"
        );

        let expected_impl: syn::ItemImpl = parse_quote! {
            impl CDrop for MyTraitVTable {
                fn drop(repr: CRefMut<Self>) {
                    unsafe { (repr.get_vtable().drop)(repr) }
                }
            }
        };
        assert_eq!(output.items[2], syn::Item::Impl(expected_impl.clone()));

        let expected_methods: syn::ItemImpl = parse_quote! {
        impl MyTraitVTable {
            /// Creates a new vtable for the type GEN that implements the trait
            pub fn new_boxed<GEN: MyTrait + 'static>(input: GEN) -> CRepr<MyTraitVTable> {
                let vtable  = MyTraitVTable::create_vtable::<GEN>();
                CRepr::new_boxed(vtable, input)
            }

            /// Creates a new vtable for the type GEN then store in a static variable in the heap
            pub fn create_vtable<GEN: MyTrait + 'static>() -> &'static MyTraitVTable {
                   static FN_MAP: std::sync::LazyLock<std::sync::Mutex<std::collections::HashMap<std::any::TypeId, &'static (dyn std::any::Any + Send + Sync)>>> =
                        std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

                    let type_id = std::any::TypeId::of::<GEN>();

                    let mut map = FN_MAP.lock().unwrap();
                    let entry = map.entry(type_id).or_insert_with(|| {
                        let vtable = Box::new(MyTraitVTable {
                            my_method: {
                                unsafe extern "C" fn my_method<GEN: MyTrait>(arg0: CRef<MyTraitVTable>, arg1: i32) -> i32 {
                                    #[allow(unsafe_code)]
                                    unsafe {
                                        GEN::my_method(&*(arg0.as_ptr() as *const GEN), arg1)
                                    }
                                }
                                my_method::<GEN>
                            },
                            another_method: {
                                unsafe extern "C" fn another_method<GEN: MyTrait>(arg0: CRefMut<MyTraitVTable>, arg1: String) {
                                    #[allow(unsafe_code)]
                                    unsafe {
                                        GEN::another_method(&mut *(arg0.as_ptr() as *mut GEN), arg1)
                                    }
                                }
                                another_method::<GEN>
                            },
                            drop: {
                                unsafe extern "C" fn drop<GEN: MyTrait>(arg_0: CRefMut<MyTraitVTable>) {
                                    #[allow(unsafe_code)]
                                    unsafe {
                                        ::core::mem::drop(Box::from_raw(arg_0.as_ptr() as *mut GEN));
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
        };
        assert_eq!(output.items[3], syn::Item::Impl(expected_methods.clone()));

        let expected_trait_for_cref: syn::ItemImpl = parse_quote! {
            impl MyTrait for CRepr<MyTraitVTable> {
                fn my_method(&self, x: i32) -> i32 {
                    #[allow(unsafe_code)]
                    unsafe {
                        (self.get_vtable().my_method)(self.as_cref(), x)
                    }
                }
                fn another_method(&mut self, y: String) {
                    #[allow(unsafe_code)]
                    unsafe {
                        (self.get_vtable().another_method)(self.as_cref_mut(), y)
                    }
                }
            }
        };
        assert_eq!(
            output.items[4],
            syn::Item::Impl(expected_trait_for_cref.clone())
        );

        let expected_trait_for_cref_where_as_vtable: syn::ItemImpl = parse_quote! {
                impl<GEN> MyTrait for CRepr<GEN>
                where
                    GEN: AsVTable<&'static MyTraitVTable> + CDrop,
                {
                    fn my_method(&self, x: i32) -> i32 {
                        let methods: &'static MyTraitVTable = self.as_vtable();
                        #[allow(unsafe_code)]
                        unsafe {
                            (methods
                                .my_method)(
                                self.as_cref_with_methods(std::ptr::NonNull::from(methods)),
                                x,
                            )
                        }
                    }
                    fn another_method(&mut self, y: String) {
                        let methods: &'static MyTraitVTable = self.as_vtable();
                        #[allow(unsafe_code)]
                        unsafe {
                            (methods
                                .another_method)(
                                self.as_cref_mut_with_methods(std::ptr::NonNull::from(methods)),
                                y,
                            )
                        }
                    }
                }
        };
        assert_eq!(
            item_to_pretty_string(output.items[5].clone()),
            item_to_pretty_string(syn::Item::Impl(
                expected_trait_for_cref_where_as_vtable.clone()
            )),
            "Generated impl for trait for CRepr with where clause does not match",
        );
    }

    #[test]
    fn test_get_super_traits() {
        let input: syn::ItemTrait = parse_quote! {
            trait MyTrait<T>: SuperTrait1 + SuperTrait2<T> + Send {
                fn my_method(&self, value: T);
            }
        };

        let output = impl_crusty_trait(input.clone());

        assert_eq!(output.items[0], syn::Item::Trait(input));

        let expected_super_trait1: syn::ItemImpl = parse_quote! {
            impl<T> AsVTable<&'static SuperTrait1VTable> for MyTraitVTable<T> {
                fn as_vtable(&self) -> &'static SuperTrait1VTable {
                    &self.field_super_trait1
                }
            }
        };

        let expected_as_vtable_1 = item_to_pretty_string(output.items[2].clone());
        let expected_as_vtable_1_ref =
            item_to_pretty_string(syn::Item::Impl(expected_super_trait1.clone()));

        assert_eq!(
            expected_as_vtable_1, expected_as_vtable_1_ref,
            "Generated impl for SuperTrait1 does not match expected: got \n{} \nexpected \n{}",
            expected_as_vtable_1, expected_as_vtable_1_ref
        );

        let expected_super_trait2: syn::ItemImpl = parse_quote! {
            impl<T> AsVTable<&'static SuperTrait2VTable<T>> for MyTraitVTable<T> {
                fn as_vtable(&self) -> &'static SuperTrait2VTable<T> {
                    &self.field_super_trait2
                }
            }
        };

        let expected_as_vtable_2 = item_to_pretty_string(output.items[3].clone());
        let expected_as_vtable_2_ref =
            item_to_pretty_string(syn::Item::Impl(expected_super_trait2.clone()));

        assert_eq!(
            expected_as_vtable_2, expected_as_vtable_2_ref,
            "Generated impl for SuperTrait2 does not match expected: got \n{} \nexpected \n{}",
            expected_as_vtable_2, expected_as_vtable_2_ref
        );

        let expected_impl: syn::ItemImpl = parse_quote! {
            impl<T> CDrop for MyTraitVTable<T> {
                fn drop(repr: CRefMut<Self>) {
                    unsafe { (repr.get_vtable().drop)(repr) }
                }
            }
        };
        assert_eq!(output.items[4], syn::Item::Impl(expected_impl.clone()));
    }
}
