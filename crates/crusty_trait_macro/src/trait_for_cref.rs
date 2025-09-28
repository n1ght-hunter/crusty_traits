use quote::{ToTokens, quote};
use syn::{Ident, Type, parse_quote, spanned::Spanned};

pub fn impl_trait_for_c_ref(input: &syn::ItemTrait, vtable: &syn::ItemStruct) -> syn::ItemImpl {
    let trait_ident = &input.ident;
    let vtable_ident = &vtable.ident;
    let generics = &input.generics;
    let methods = input
        .items
        .clone()
        .into_iter()
        .filter_map(|i| {
            if let syn::TraitItem::Fn(fn_item) = i {
                Some(fn_item)
            } else {
                None
            }
        })
        .map(|mut f| {
            let method_name = &f.sig.ident;
            f.semi_token = None;

            let inputs = f.sig.inputs.iter().map(|input| match input {
                syn::FnArg::Receiver(recv) => {
                    let ty = recv.ty.as_ref().clone();

                    match ty {
                        Type::Reference(type_ref) if type_ref.mutability.is_none() => {
                            quote! {
                                self.as_cref()
                            }
                        }
                        Type::Reference(_) => {
                            quote! {
                                self.as_cref_mut()
                            }
                        }
                        _ => syn::Error::new(ty.span(), "Receiver type must be a reference")
                            .to_compile_error(),
                    }
                }
                syn::FnArg::Typed(pat_type) => {
                    let name = pat_type.pat.as_ref().clone();
                    quote! {
                        #name
                    }
                }
            });

            f.default = Some(parse_quote!({
                #[allow(unsafe_code)]
                unsafe {
                    (self.get_vtable().#method_name)(#(#inputs),*)
                }
            }));

            quote! {
                #f
            }
        });

    parse_quote! {
        impl #generics #trait_ident #generics for CRepr<#vtable_ident  #generics> {
            #(#methods)*
        }
    }
}

pub fn impl_trait_for_c_ref_where_as_vtable(
    input: &syn::ItemTrait,
    vtable: &syn::ItemStruct,
    super_traits: &crate::super_trait::SuperTraits,
    ignore_bounds: &Vec<String>,
) -> syn::ItemImpl {
    let trait_ident = &input.ident;
    let vtable_ident = &vtable.ident;
    let generics = &input.generics;

    let methods = input
        .items
        .clone()
        .into_iter()
        .filter_map(|i| {
            if let syn::TraitItem::Fn(fn_item) = i {
                Some(fn_item)
            } else {
                None
            }
        })
        .map(|mut f| {
            let method_name = &f.sig.ident;
            f.semi_token = None;

            let inputs = f.sig.inputs.iter().map(|input| match input {
                syn::FnArg::Receiver(recv) => {
                    let ty = recv.ty.as_ref().clone();

                    match ty {
                        Type::Reference(type_ref) if type_ref.mutability.is_none() => {
                            quote! {
                                self.as_cref_with_methods(std::ptr::NonNull::from(methods))
                            }
                        }
                        Type::Reference(_) => {
                            quote! {
                                self.as_cref_mut_with_methods(std::ptr::NonNull::from(methods))
                            }
                        }
                        _ => syn::Error::new(ty.span(), "Receiver type must be a reference")
                            .to_compile_error(),
                    }
                }
                syn::FnArg::Typed(pat_type) => {
                    let name = pat_type.pat.as_ref().clone();
                    quote! {
                        #name
                    }
                }
            });

            f.default = Some(parse_quote!({
                let methods: &'static #vtable_ident #generics = self.as_vtable();
                #[allow(unsafe_code)]
                unsafe {
                    (methods.#method_name)(#(#inputs),*)
                }
            }));

            f.to_token_stream()
        });

    let super_trait_as_vtable = super_traits.iter().map(|super_trait| {
        let Type::Reference(vtable_ref) = &super_trait.vtable_ty else {
            return syn::Error::new(
                super_trait.vtable_ty.span(),
                "Super trait vtable type must be a reference",
            )
            .to_compile_error();
        };
        let super_trait_ident = vtable_ref.elem.as_ref();

        quote! { + AsVTable<&'static #super_trait_ident> }
    });

    let ignore_bounds = ignore_bounds.iter().map(|bound| {
        let bound_ident = Ident::new(bound, proc_macro2::Span::call_site());
        quote! { + #bound_ident }
    });

    let mut start_gen = generics.clone();

    start_gen.params.insert(0, parse_quote!(GEN));

    let static_generics = generics.params.iter().map(|param| {
        if let syn::GenericParam::Type(ty) = param {
            let ident = &ty.ident;
            quote! { #ident: 'static }
        } else {
            quote! {}
        }
    });

    parse_quote! {
        impl #start_gen #trait_ident #generics for CRepr<GEN>
        where
            GEN: AsVTable<&'static #vtable_ident #generics> + CDrop  #(#super_trait_as_vtable)* #(#ignore_bounds)*,
            #(#static_generics),*
        {
            #(#methods)*
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::super_trait::SuperTraitReturn;

    use super::*;

    #[test]
    fn impl_trait_for_cref_test_where_as_vtable() {
        let input: syn::ItemTrait = parse_quote! {
            pub trait MyTrait: SuperTrait {
                fn my_method(&self, x: i32) -> i32;
                fn another_method(&mut self, y: String);
            }
        };
        let vtable: syn::ItemStruct = parse_quote! {
            pub struct MyTraitVTable {
                pub my_method: unsafe fn(this: &CRepr<MyTraitVTable>, x: i32) -> i32,
                pub another_method: unsafe fn(this: &mut CRepr<MyTraitVTable>, y: String),
                pub field_super_trait: SuperTraitVTable,
            }
        };
        let SuperTraitReturn {
            super_traits,
            ignore_bounds,
        } = crate::super_trait::get_super_traits(&input).unwrap();
        let output =
            impl_trait_for_c_ref_where_as_vtable(&input, &vtable, &super_traits, &ignore_bounds);

        let expected_output: syn::ItemImpl = parse_quote!(
            impl<GEN> MyTrait for CRepr<GEN>
            where
                GEN: AsVTable<&'static MyTraitVTable> + CDrop + AsVTable<&'static SuperTraitVTable>,
            {
                fn my_method(&self, x: i32) -> i32 {
                    let methods: &'static MyTraitVTable = self.as_vtable();
                    #[allow(unsafe_code)]
                    unsafe {
                        (methods.my_method)(
                            self.as_cref_with_methods(std::ptr::NonNull::from(methods)),
                            x,
                        )
                    }
                }
                fn another_method(&mut self, y: String) {
                    let methods: &'static MyTraitVTable = self.as_vtable();
                    #[allow(unsafe_code)]
                    unsafe {
                        (methods.another_method)(
                            self.as_cref_mut_with_methods(std::ptr::NonNull::from(methods)),
                            y,
                        )
                    }
                }
            }
        );
        assert_eq!(
            crate::utils::test_utils::item_to_pretty_string(syn::Item::Impl(output.clone())),
            crate::utils::test_utils::item_to_pretty_string(syn::Item::Impl(expected_output))
        );
    }

    #[test]
    fn impl_trait_for_cref_test() {
        let input: syn::ItemTrait = parse_quote! {
            pub trait MyTrait {
                fn my_method(&self, x: i32) -> i32;
                fn another_method(&mut self, y: String);
            }
        };
        let vtable: syn::ItemStruct = parse_quote! {
            pub struct MyTraitVTable {
                pub my_method: unsafe fn(this: &CRepr<MyTraitVTable>, x: i32) -> i32,
                pub another_method: unsafe fn(this: &mut CRepr<MyTraitVTable>, y: String),
            }
        };
        let output = impl_trait_for_c_ref(&input, &vtable);

        let expected_output: syn::ItemImpl = parse_quote!(
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
        );

        assert_eq!(output, expected_output);
    }

    #[test]
    fn impl_trait_for_cref_test_with_generics() {
        let input: syn::ItemTrait = parse_quote! {
            pub trait MyTrait<T> {
                fn my_method(&self, x: T) -> T;
                fn another_method(&mut self, y: String);
            }
        };
        let vtable: syn::ItemStruct = parse_quote! {
            pub struct MyTraitVTable<T> {
                pub my_method: unsafe fn(this: &CRepr<MyTraitVTable<T>>, x: T) -> T,
                pub another_method: unsafe fn(this: &mut CRepr<MyTraitVTable<T>>, y: String),
            }
        };
        let output = impl_trait_for_c_ref(&input, &vtable);

        let expected_output: syn::ItemImpl = parse_quote!(
            impl<T> MyTrait<T> for CRepr<MyTraitVTable<T>> {
                fn my_method(&self, x: T) -> T {
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
        );

        assert_eq!(output, expected_output);
    }

    #[test]
    fn impl_trait_for_cref_test_with_generics_and_supertrait() {
        let input: syn::ItemTrait = parse_quote! {
            pub trait MyTrait<T>: SuperTrait<T> {
                fn my_method(&self, x: T) -> T;
                fn another_method(&mut self, y: String);
            }
        };
        let vtable: syn::ItemStruct = parse_quote! {
            pub struct MyTraitVTable<T> {
                pub my_method: unsafe fn(this: &CRepr<MyTraitVTable<T>>, x: T) -> T,
                pub another_method: unsafe fn(this: &mut CRepr<MyTraitVTable<T>>, y: String),
                pub field_super_trait: SuperTraitVTable<T>,
            }
        };

        let output = impl_trait_for_c_ref(&input, &vtable);
        let expected_output: syn::ItemImpl = parse_quote!(
            impl<T> MyTrait<T> for CRepr<MyTraitVTable<T>> {
                fn my_method(&self, x: T) -> T {
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
        );

        assert_eq!(output, expected_output);
    }
}
