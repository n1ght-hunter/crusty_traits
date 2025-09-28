use quote::{ToTokens, format_ident, quote};
use syn::{
    GenericParam, Ident, ItemStruct, ItemTrait, TraitItem, Type, TypeParamBound, parse_quote,
};

use crate::{
    utils::{self, map_genrics_ident},
    vtable::map_inputs,
};

pub fn impl_vtable_methods(input: &ItemTrait, vtable: &ItemStruct) -> syn::Item {
    let genrics = &input.generics;
    let trait_ident = &input.ident;
    let vtable_ident = &vtable.ident;
    let mut renamed_generics = genrics.clone();

    let mut static_generics = genrics.clone();

    static_generics.params.iter_mut().for_each(|param| {
        if let GenericParam::Type(type_param) = param {
            if !type_param.bounds.iter().any(|bound| {
                if let TypeParamBound::Lifetime(lifetime) = bound {
                    lifetime.ident == "static"
                } else {
                    false
                }
            }) {
                type_param.bounds.push(parse_quote!('static));
            }
        }
    });

    renamed_generics.params.iter_mut().for_each(|g| {
        map_genrics_ident(g, &utils::map_method_ident);
    });

    let mut method_generics = renamed_generics.clone();

    method_generics
        .params
        .push(parse_quote!(GEN: #trait_ident #renamed_generics));

    let mut method_generics_names = genrics
        .params
        .iter()
        .map(|p| match p {
            GenericParam::Lifetime(lifetime) => {
                let ident = &lifetime.lifetime.ident;
                quote! { #ident }
            }
            GenericParam::Type(type_param) => {
                let ident = &type_param.ident;
                quote! { #ident }
            }
            GenericParam::Const(const_param) => {
                let ident = &const_param.ident;
                quote! { #ident }
            }
        })
        .collect::<Vec<_>>();

    method_generics_names.push(Ident::new("GEN", proc_macro2::Span::call_site()).to_token_stream());

    let mapper = |ty: &mut syn::TypePath| {
        if genrics.params.iter().any(|p| {
            if let GenericParam::Type(type_param) = p
                && let Some(ty_path) = ty.path.get_ident()
            {
                &type_param.ident == ty_path
            } else {
                false
            }
        }) {
            ty.path.segments.first_mut().unwrap().ident =
                utils::map_method_ident(ty.path.segments.first_mut().unwrap().ident.clone());
        }
    };

    let map_genrics = |param: &mut Type| {
        if let Type::Path(type_path) = param {
            mapper(type_path);
        }
    };

    let mut methods = input
        .items
        .iter()
        .filter_map(|i| {
            if let TraitItem::Fn(fn_item) = i {
                Some(fn_item)
            } else {
                None
            }
        })
        .map(|f| {
            let method_name = &f.sig.ident;
            let inputs = map_inputs(
                &f.sig.inputs,
                Some(quote! { #vtable_ident #renamed_generics}),
            )
            .enumerate()
            .map(|(i, mut arg)| {
                let name = format_ident!("arg{}", i);
                arg.name = Some((name, Default::default()));
                utils::map_ty(&mut arg.ty, &mapper);
                arg
            })
            .collect::<Vec<_>>();

            let pass_in_args = inputs
                .clone()
                .into_iter()
                .map(|arg| {
                    let name = arg.name.unwrap().0;

                    if let Type::Path(path) = &arg.ty {
                        let path = path.path.segments.first().map(|s| s.ident.to_string());
                        match path.as_ref().map(|s| s.as_str()) {
                            Some("CRef") => {
                                quote! {
                                    &*(#name.as_ptr() as *const GEN)
                                }
                            }
                            Some("CRefMut") => {
                                quote! {
                                    &mut *(#name.as_ptr() as *mut GEN)
                                }
                            }
                            _ => {
                                quote! {
                                    #name
                                }
                            }
                        }
                    } else {
                        quote! {
                            #name
                        }
                    }
                })
                .collect::<Vec<_>>();

            let mut output = f.sig.output.clone();

            if let syn::ReturnType::Type(_, ref mut ty) = output {
                utils::map_ty(ty, &mapper);
                utils::map_ty_genrics(ty, &map_genrics);
            }

            quote! {
                #method_name: {
                    unsafe extern "C" fn #method_name #method_generics(
                        #(#inputs),*
                    ) #output {
                        #[allow(unsafe_code)]
                        unsafe {
                            GEN::#method_name(
                                #(#pass_in_args),*
                            )
                        }
                    }
                    #method_name::<#(#method_generics_names),*>
                }
            }
        })
        .collect::<Vec<_>>();

    let super_trait_field = vtable
        .fields
        .iter()
        .filter_map(|field| {
            if let Type::Reference(ty) = &field.ty {
                if let Type::Path(path) = &*ty.elem {
                    let ident = field.ident.as_ref().unwrap();
                    let super_vtable_ty = path.path.segments.first().unwrap().ident.clone();
                    return Some(quote! {
                        #ident: #super_vtable_ty::create_vtable::<GEN>()
                    });
                }
            }

            None
        })
        .collect::<Vec<_>>();

    methods.extend(super_trait_field);

    let methods = if methods.len() > 0 {
        quote! {
            #(#methods),*,
        }
    } else {
        quote! {}
    };

    let vtable_creater = quote! {
    #vtable_ident {
        #methods

       drop: {
           unsafe extern "C" fn drop #method_generics(arg_0: CRefMut<#vtable_ident #renamed_generics>) {
               #[allow(unsafe_code)]
               unsafe {
                   ::core::mem::drop(Box::from_raw(arg_0.as_ptr() as *mut GEN));
               }
           }
           drop::<#(#method_generics_names),*>
       },
      }};

    let mut genrics_params = renamed_generics.clone();

    genrics_params
        .params
        .push(parse_quote!(GEN: #trait_ident #renamed_generics + 'static));

    parse_quote! {
        impl #static_generics #vtable_ident #genrics {
            /// Creates a new vtable for the type GEN that implements the trait
            pub fn new_boxed<GEN: #trait_ident #genrics + 'static>(input: GEN) -> CRepr<#vtable_ident #genrics> {
                let vtable  = #vtable_ident::create_vtable::<GEN>();
                CRepr::new_boxed(vtable, input)
            }

            /// Creates a new vtable for the type GEN then store in a static variable in the heap
            pub fn create_vtable<GEN: #trait_ident #genrics + 'static>() -> &'static #vtable_ident #genrics {
                   static FN_MAP: std::sync::LazyLock<std::sync::Mutex<std::collections::HashMap<std::any::TypeId, &'static (dyn std::any::Any + Send + Sync)>>> =
                        std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

                    let type_id = std::any::TypeId::of::<GEN>();

                    let mut map = FN_MAP.lock().unwrap();
                    let entry = map.entry(type_id).or_insert_with(|| {
                        let vtable = Box::new(#vtable_creater);
                        Box::leak(vtable)
                    });
                    entry.downcast_ref().unwrap()


            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_impl_vtable_methods() {
        let input: ItemTrait = parse_quote! {
            trait MyTrait<T> {
                fn my_method(&self, value: T) -> T;
            }
        };

        let vtable: ItemStruct = parse_quote! {
            struct MyTraitVTable<T> {
                my_method: unsafe extern "C" fn(arg0: CRef<MyTraitVTable<T>>, arg1: T) -> T,
                drop: unsafe extern "C" fn(arg0: CRefMut<MyTraitVTable<T>>),
            }
        };

        let result = impl_vtable_methods(&input, &vtable);

        let expected: syn::ItemImpl = parse_quote! {
        impl<T: 'static> MyTraitVTable<T> {
            /// Creates a new vtable for the type GEN that implements the trait
            pub fn new_boxed<GEN: MyTrait<T> + 'static>(input: GEN) -> CRepr<MyTraitVTable<T>> {
                let vtable  = MyTraitVTable::create_vtable::<GEN>();
                CRepr::new_boxed(vtable, input)
            }

            /// Creates a new vtable for the type GEN then store in a static variable in the heap
            pub fn create_vtable<GEN: MyTrait<T> + 'static>() -> &'static MyTraitVTable<T> {
                   static FN_MAP: std::sync::LazyLock<std::sync::Mutex<std::collections::HashMap<std::any::TypeId, &'static (dyn std::any::Any + Send + Sync)>>> =
                        std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

                    let type_id = std::any::TypeId::of::<GEN>();

                    let mut map = FN_MAP.lock().unwrap();
                    let entry = map.entry(type_id).or_insert_with(|| {
                        let vtable = Box::new(MyTraitVTable {
                            my_method: {
                                unsafe extern "C" fn my_method<TMETHOD, GEN: MyTrait<TMETHOD>>(arg0: CRef<MyTraitVTable<TMETHOD>>, arg1: TMETHOD) -> TMETHOD {
                                    #[allow(unsafe_code)]
                                    unsafe {
                                        GEN::my_method(&*(arg0.as_ptr() as *const GEN), arg1)
                                    }
                                }
                                my_method::<T, GEN>
                            },
                            drop: {
                                unsafe extern "C" fn drop<TMETHOD, GEN: MyTrait<TMETHOD>>(arg_0: CRefMut<MyTraitVTable<TMETHOD>>) {
                                    #[allow(unsafe_code)]
                                    unsafe {
                                        ::core::mem::drop(Box::from_raw(arg_0.as_ptr() as *mut GEN));
                                    }
                                }
                                drop::<T, GEN>
                            },
                        });
                        Box::leak(vtable)
                    });
                    entry.downcast_ref().unwrap()
                }
            }
        };
        let string_expected =
            utils::test_utils::item_to_pretty_string(syn::Item::Impl(expected.clone()));
        let string_result = utils::test_utils::item_to_pretty_string(result.clone());
        assert_eq!(
            string_result, string_expected,
            "Generated impl does not match expected impl: expected:\n{}\n\nGot:\n{}",
            string_expected, string_result
        );
    }
}
