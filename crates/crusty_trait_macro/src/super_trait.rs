use quote::quote;
use syn::{Ident, ItemTrait, Type, parse_quote};

use crate::{
    IGNORE_SUPER_TRAITS,
    utils::{map_field_ident, map_vec_to_generics, map_vtable_ident},
};

pub struct SuperTrait {
    pub ident: Ident,
    pub vtable_ty: Type,
    pub field_ident: Ident,
    pub path: syn::Path,
    pub generics: Vec<Ident>,
}

pub type SuperTraits = Vec<SuperTrait>;

pub struct SuperTraitReturn {
    pub super_traits: SuperTraits,
    pub ignore_bounds: Vec<String>,
}

pub fn get_super_traits(input: &ItemTrait) -> syn::Result<SuperTraitReturn> {
    let mut ignore_bounds = vec![];
    let mut errors = vec![];
    let super_traits = input
        .supertraits
        .iter()
        .filter_map(|s| match s {
            syn::TypeParamBound::Trait(trait_bound) => {
                let Some(ident) = trait_bound.path.segments.first().map(|s| s.ident.clone()) else {
                    errors.push(syn::Error::new_spanned(
                        trait_bound,
                        "Only simple traits are supported as super traits",
                    ));
                    return None;
                };
                if IGNORE_SUPER_TRAITS.contains(&ident.to_string().as_str()) {
                    ignore_bounds.push(ident.to_string());
                    return None;
                }
                Some((trait_bound, ident))
            }
            _ => None,
        })
        .map(|(trait_bound, ident)| {
            let vtable_ident = map_vtable_ident(ident.clone());
            let field_ident = map_field_ident(ident.clone());

            let mut generics = vec![];
            if let Some(segment) = trait_bound.path.segments.first()
                && let syn::PathArguments::AngleBracketed(angle_bracketed) = &segment.arguments
            {
                for arg in &angle_bracketed.args {
                    if let syn::GenericArgument::Type(ty) = arg {
                        if let Type::Path(type_path) = ty {
                            if let Some(ident) =
                                type_path.path.segments.first().map(|s| s.ident.clone())
                            {
                                generics.push(ident);
                            }
                        }
                    }
                }
            }

            let vtable_ty = Type::Reference(syn::TypeReference {
                and_token: Default::default(),
                lifetime: Some(syn::Lifetime {
                    apostrophe: proc_macro2::Span::call_site(),
                    ident: Ident::new("static", proc_macro2::Span::call_site()),
                }),
                mutability: None,
                elem: Box::new(Type::Path(syn::TypePath {
                    qself: None,
                    path: syn::Path {
                        leading_colon: None,
                        segments: syn::punctuated::Punctuated::from_iter(vec![syn::PathSegment {
                            ident: vtable_ident,
                            arguments: if generics.is_empty() {
                                syn::PathArguments::None
                            } else {
                                syn::PathArguments::AngleBracketed(
                                    parse_quote!(< #( #generics ),* >),
                                )
                            },
                        }]),
                    },
                })),
            });

            SuperTrait {
                ident: ident.clone(),
                field_ident,
                path: trait_bound.path.clone(),
                vtable_ty,
                generics,
            }
        })
        .collect::<Vec<_>>();

    let errors = errors
        .into_iter()
        .fold(None, |acc: Option<syn::Error>, err: syn::Error| {
            if let Some(mut acc) = acc {
                acc.combine(err.clone());
                return Some(acc);
            } else {
                return Some(err.clone());
            }
        });

    if let Some(errors) = errors {
        return Err(errors);
    }

    Ok(SuperTraitReturn {
        super_traits,
        ignore_bounds,
    })
}

pub fn impl_as_vtable_for_super_traits(
    super_traits: &SuperTraits,
    vtable: &syn::ItemStruct,
) -> impl Iterator<Item = syn::Item> {
    super_traits.iter().map(move |super_trait| {
        let field_ident = &super_trait.field_ident;
        let vtable_ty = &super_trait.vtable_ty;
        let generics = if super_trait.generics.is_empty() && vtable.generics.params.is_empty() {
            quote! {}
        } else {
            let mut generics = vtable
                .generics
                .params
                .iter()
                .filter_map(|param| {
                    if let syn::GenericParam::Type(type_param) = param {
                        Some(type_param.ident.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            generics.extend(super_trait.generics.iter().cloned());
            generics.sort();
            generics.dedup();
            quote! { <#(#generics),*> }
        };

        let vtable_ident = &vtable.ident;

        syn::parse_quote! {
                impl #generics AsVTable<#vtable_ty> for #vtable_ident #generics {
                fn as_vtable(&self) -> #vtable_ty {
                    &self.#field_ident
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_get_super_traits() {
        let item: ItemTrait = parse_quote! {
            trait MyTrait: SuperTrait1 + SuperTrait2 + Send {
                fn my_method(&self);
            }
        };

        let result = get_super_traits(&item).unwrap();
        assert_eq!(result.super_traits.len(), 2);
        assert_eq!(result.super_traits[0].ident, "SuperTrait1");
        assert_eq!(result.super_traits[1].ident, "SuperTrait2");
        assert_eq!(result.ignore_bounds, vec!["Send"]);
    }

    #[test]
    fn test_get_super_traits_with_generic() {
        let item: ItemTrait = parse_quote! {
            trait MyTrait<T>: SuperTrait1 + Sized {
                fn my_method(&self, value: T);
            }
        };

        let result = get_super_traits(&item).unwrap();
        assert_eq!(result.super_traits.len(), 1);
        assert_eq!(result.super_traits[0].ident, "SuperTrait1");
        assert_eq!(result.ignore_bounds, vec!["Sized"]);
    }

    #[test]
    fn test_get_super_traits_no_super_traits() {
        let item: ItemTrait = parse_quote! {
            trait MyTrait {
                fn my_method(&self);
            }
        };

        let result = get_super_traits(&item).unwrap();
        assert_eq!(result.super_traits.len(), 0);
        assert_eq!(result.ignore_bounds.len(), 0);
    }

    #[test]
    fn test_with_super_trait_generic() {
        let item: ItemTrait = parse_quote! {
            trait MyTrait<T>: SuperTrait1<T> + Send {
                fn my_method(&self, value: T);
            }
        };

        let result = get_super_traits(&item).unwrap();
        assert_eq!(result.super_traits.len(), 1);
        assert_eq!(result.super_traits[0].ident, "SuperTrait1");
        assert_eq!(
            result.super_traits[0].generics,
            vec![Ident::new("T", proc_macro2::Span::call_site())]
        );
        assert_eq!(result.ignore_bounds, vec!["Send"]);

        let item: ItemTrait = parse_quote! {
            trait MyTrait<T, U>: SuperTrait1<T, U> + Send {
                fn my_method(&self, value: T, other: U);
            }
        };

        let result = get_super_traits(&item).unwrap();
        assert_eq!(result.super_traits.len(), 1);
        assert_eq!(result.super_traits[0].ident, "SuperTrait1");
        assert_eq!(
            result.super_traits[0].generics,
            vec![
                Ident::new("T", proc_macro2::Span::call_site()),
                Ident::new("U", proc_macro2::Span::call_site())
            ]
        );
        assert_eq!(result.ignore_bounds, vec!["Send"]);
    }

    #[test]
    fn test_impl_as_vtable_for_super_traits() {
        let vtable: syn::ItemStruct = parse_quote! {
            struct MyTraitVTable<T> {
                field_super_trait1: &'static SuperTrait1VTable,
                field_super_trait2: &'static SuperTrait2VTable<T>,
            }
        };

        let super_traits = get_super_traits(&parse_quote! {
            trait MyTrait<T>: SuperTrait1 + SuperTrait2<T> + Send {
                fn my_method(&self, value: T);
            }
        })
        .unwrap()
        .super_traits;
        let impls = impl_as_vtable_for_super_traits(&super_traits, &vtable).collect::<Vec<_>>();

        let expected_1: syn::ItemImpl = parse_quote! {
            impl<T> AsVTable<&'static SuperTrait1VTable> for MyTraitVTable<T> {
                fn as_vtable(&self) -> &'static SuperTrait1VTable {
                    &self.field_super_trait1
                }
            }
        };

        let expected_2: syn::ItemImpl = parse_quote! {
            impl<T> AsVTable<&'static SuperTrait2VTable<T>> for MyTraitVTable<T> {
                fn as_vtable(&self) -> &'static SuperTrait2VTable<T> {
                    &self.field_super_trait2
                }
            }
        };

        assert_eq!(impls.len(), 2);
        assert_eq!(impls[0], syn::Item::Impl(expected_1));
        assert_eq!(impls[1], syn::Item::Impl(expected_2));
    }
}
