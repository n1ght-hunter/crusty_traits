//! macro to impl the crusty traits.

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use super_trait::{SuperTraits, get_super_traits};
use syn::{
    BareFnArg, Field, ItemStruct, LitStr, Token, TraitItem, Type, TypeBareFn, Visibility,
    parse_quote, token::At,
};
use utils::{doc_attribute, repr_c_attribute};

mod cdrop;
mod super_trait;
mod utils;

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

    output.items.push(input.into());
    output.items.push(vtable.into());
    output.items.extend(as_vtable_impls);

    output
}

fn create_vtable(
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

    let super_trait_fields = super_traits
        .iter()
        .map(|super_trait| Field {
            attrs: vec![],
            vis: Visibility::Public(Default::default()),
            mutability: syn::FieldMutability::None,
            ident: Some(super_trait.field_ident.clone()),
            colon_token: Some(Default::default()),
            ty: super_trait.vtable_ty.clone(),
        })
        .collect::<Vec<_>>();

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
                pub my_method: unsafe extern "C" fn(CRef<Self>, i32) -> i32,
                pub another_method: unsafe extern "C" fn(CRefMut<Self>, String),
                #[doc = "A function pointer to the drop function for the trait"]
                pub drop: unsafe extern "C" fn(CRefMut<Self>),
            }
        };

        // Compare using pretty-printed representation to avoid structural metadata issues
        let generated_vtable = item_to_pretty_string(output.items[1].clone());
        let expected_vtable = item_to_pretty_string(syn::Item::Struct(expected_vtable.clone()));

        assert_eq!(
            generated_vtable, expected_vtable,
            "Generated vtable does not match expected vtable"
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
    }
}
