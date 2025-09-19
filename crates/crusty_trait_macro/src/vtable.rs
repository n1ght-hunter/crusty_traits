pub mod methods;

use proc_macro2::TokenStream;
use quote::quote;

use syn::{
    BareFnArg, Field, ItemStruct, LitStr, Token, TraitItem, Type, TypeBareFn, Visibility,
    parse_quote,
};

use crate::{
    super_trait::SuperTraits,
    utils::{self, doc_attribute, repr_c_attribute},
};

pub fn create_vtable(
    input: &syn::ItemTrait,
    super_traits: &SuperTraits,
) -> Result<ItemStruct, syn::Error> {
    let trait_ident = &input.ident;
    let repr_c = repr_c_attribute();
    let string_ident = trait_ident.to_string();
    let docs = doc_attribute(format!("A repr C vtable for the trait {}", string_ident));
    let mut vtable = ItemStruct {
        attrs: vec![repr_c, docs],
        vis: input.vis.clone(),
        struct_token: syn::token::Struct {
            span: proc_macro2::Span::call_site(),
        },
        ident: utils::map_vtable_ident(trait_ident.clone()),
        generics: input.generics.clone(),
        fields: syn::Fields::Named(syn::FieldsNamed {
            brace_token: syn::token::Brace::default(),
            named: syn::punctuated::Punctuated::new(),
        }),
        semi_token: None,
    };

    let mut fields = input
        .items
        .iter()
        .filter_map(|i| {
            if let TraitItem::Fn(fn_item) = i {
                Some(fn_item)
            } else {
                None
            }
        })
        .map(|method| {
            let ty = TypeBareFn {
                lifetimes: None,
                unsafety: Some(Default::default()),
                abi: Some(syn::Abi {
                    extern_token: Default::default(),
                    name: Some(LitStr::new("C", proc_macro2::Span::call_site())),
                }),
                fn_token: Default::default(),
                paren_token: Default::default(),
                inputs: map_inputs(&method.sig.inputs, None).collect(),
                variadic: None,
                output: method.sig.output.clone(),
            };
            Field {
                attrs: method.attrs.clone(),
                vis: Visibility::Public(Default::default()),
                mutability: syn::FieldMutability::None,
                ident: Some(method.sig.ident.clone()),
                colon_token: Some(Default::default()),
                ty: Type::BareFn(ty),
            }
        })
        .collect::<Vec<_>>();

    let mut needs_statlic = Vec::new();

    let super_trait_fields = super_traits
        .iter()
        .map(|super_trait| {
            needs_statlic.extend(super_trait.generics.iter().cloned());
            Field {
                attrs: vec![],
                vis: Visibility::Public(Default::default()),
                mutability: syn::FieldMutability::None,
                ident: Some(super_trait.field_ident.clone()),
                colon_token: Some(Default::default()),
                ty: super_trait.vtable_ty.clone(),
            }
        })
        .collect::<Vec<_>>();

    needs_statlic.sort();
    needs_statlic.dedup();
    needs_statlic.into_iter().for_each(|generic| {
        if let Some(param) = vtable.generics.params.iter_mut().find(|param| match param {
            syn::GenericParam::Type(type_param) => type_param.ident == generic,
            _ => false,
        }) {
            if let syn::GenericParam::Type(type_param) = param
                && !type_param.bounds.iter().any(|b| match b {
                    syn::TypeParamBound::Lifetime(lifetime) => lifetime.ident == "static",
                    _ => false,
                })
            {
                type_param.bounds.push(parse_quote!( 'static ));
            }
        }
    });

    fields.extend(super_trait_fields);

    let drop_field: Field = parse_quote!(
        #[doc = "A function pointer to the drop function for the trait"]
        pub drop: unsafe extern "C" fn(CRefMut<Self>)
    );
    fields.push(drop_field);

    vtable.fields = syn::Fields::Named(syn::FieldsNamed {
        brace_token: syn::token::Brace::default(),
        named: fields.into_iter().collect(),
    });

    Ok(vtable)
}

fn map_inputs(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, Token![,]>,
    name: Option<TokenStream>,
) -> impl Iterator<Item = BareFnArg> {
    let name = name.unwrap_or_else(|| quote! { Self });
    inputs.iter().map(move |arg: &syn::FnArg| match arg {
        syn::FnArg::Receiver(recv) => {
            let ty = recv.ty.as_ref().clone();

            match ty {
                Type::Reference(type_ref) if type_ref.mutability.is_none() => BareFnArg {
                    attrs: recv.attrs.clone(),
                    name: None,
                    ty: parse_quote!(CRef<#name>),
                },
                Type::Reference(_) => BareFnArg {
                    attrs: recv.attrs.clone(),
                    name: None,
                    ty: parse_quote!(CRefMut<#name>),
                },
                Type::Path(_) => BareFnArg {
                    attrs: recv.attrs.clone(),
                    name: None,
                    ty: parse_quote!(CRepr<#name>),
                },
                _ => {
                    panic!("Receiver type must be a reference");
                }
            }
        }
        syn::FnArg::Typed(pat_type) => BareFnArg {
            attrs: pat_type.attrs.clone(),
            name: None,
            ty: pat_type.ty.as_ref().clone(),
        },
    })
}

#[cfg(test)]
mod tests {
    use crate::utils::test_utils::{item_to_file, item_to_pretty_string};

    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_create_vtable_no_methods() {
        let input: syn::ItemTrait = parse_quote! {
            pub trait MyTrait {}
        };
        let super_traits = SuperTraits::default();
        let vtable = create_vtable(&input, &super_traits).unwrap();

        let expected: syn::ItemStruct = parse_quote! {
            #[repr(C)]
            #[doc = "A repr C vtable for the trait MyTrait"]
            pub struct MyTraitVTable {
                #[doc = "A function pointer to the drop function for the trait"]
                pub drop: unsafe extern "C" fn(CRefMut<Self>),
            }
        };

        assert_eq!(
            item_to_pretty_string(syn::Item::Struct(vtable)),
            item_to_pretty_string(syn::Item::Struct(expected))
        );
    }

    #[test]
    fn test_create_vtable_with_methods() {
        let input: syn::ItemTrait = parse_quote! {
            pub trait MyTrait {
                fn method1(&self);
                fn method2(&mut self, value: i32) -> i32;
            }
        };
        let super_traits = SuperTraits::default();
        let vtable = create_vtable(&input, &super_traits).unwrap();

        let expected: syn::ItemStruct = parse_quote! {
            #[repr(C)]
            #[doc = "A repr C vtable for the trait MyTrait"]
            pub struct MyTraitVTable {
                pub method1: unsafe extern "C" fn(CRef<Self>),
                pub method2: unsafe extern "C" fn(CRefMut<Self>, i32) -> i32,
                #[doc = "A function pointer to the drop function for the trait"]
                pub drop: unsafe extern "C" fn(CRefMut<Self>),
            }
        };

        assert_eq!(
            item_to_pretty_string(syn::Item::Struct(vtable)),
            item_to_pretty_string(syn::Item::Struct(expected))
        );
    }

    #[test]
    fn test_create_vtable_with_super_traits() {
        let input: syn::ItemTrait = parse_quote! {
            pub trait MyTrait: SuperTrait1 {
                fn method1(&self);
            }
        };
        let super_traits = crate::super_trait::get_super_traits(&input).unwrap();

        let vtable = create_vtable(&input, &super_traits.super_traits).unwrap();

        let expected: syn::ItemStruct = parse_quote! {
            #[repr(C)]
            #[doc = "A repr C vtable for the trait MyTrait"]
            pub struct MyTraitVTable {
                pub method1: unsafe extern "C" fn(CRef<Self>),
                pub field_super_trait1: &'static SuperTrait1VTable,
                #[doc = "A function pointer to the drop function for the trait"]
                pub drop: unsafe extern "C" fn(CRefMut<Self>),
            }
        };

        assert_eq!(
            item_to_pretty_string(syn::Item::Struct(vtable)),
            item_to_pretty_string(syn::Item::Struct(expected))
        );
    }

    #[test]
    fn test_create_vtable_with_super_traits_and_generics() {
        let input: syn::ItemTrait = parse_quote! {
            pub trait MyTrait<T>: SuperTrait1<T> {
                fn method1(&self, value: T);
            }
        };
        let super_traits = crate::super_trait::get_super_traits(&input).unwrap();

        let vtable = create_vtable(&input, &super_traits.super_traits).unwrap();

        let expected: syn::ItemStruct = parse_quote! {
            #[repr(C)]
            #[doc = "A repr C vtable for the trait MyTrait"]
            pub struct MyTraitVTable<T: 'static> {
                pub method1: unsafe extern "C" fn(CRef<Self>, T),
                pub field_super_trait1: &'static SuperTrait1VTable<T>,
                #[doc = "A function pointer to the drop function for the trait"]
                pub drop: unsafe extern "C" fn(CRefMut<Self>),
            }
        };

        assert_eq!(
            item_to_pretty_string(syn::Item::Struct(vtable)),
            item_to_pretty_string(syn::Item::Struct(expected))
        );
    }
}
