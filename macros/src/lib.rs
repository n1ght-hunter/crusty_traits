#![allow(missing_docs)]

use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{
    Attribute, BareFnArg, Field, ItemStruct, ItemTrait, LitStr, Token, TraitItem, TypeBareFn,
    Visibility, parse_quote, spanned::Spanned,
};

#[proc_macro_attribute]
pub fn crusty_trait(
    _attrs: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as ItemTrait);

    impl_crusty_trait(input).into()
}

fn impl_crusty_trait(input: ItemTrait) -> TokenStream {
    let mut output = input.to_token_stream();
    let vtable = create_vtable(&input);
    output.extend(
        vtable
            .as_ref()
            .map(|vtable| vtable.to_token_stream())
            .unwrap_or_else(|err| err.to_compile_error()),
    );

    if let Ok(vtable) = &vtable {
        let cdrop = impl_cdrop_for_vtable(vtable);
        output.extend(cdrop);

        let vtable_methods = impl_vtable_methods(&input, vtable);
        output.extend(vtable_methods);
        let c_ref_impl = impl_trait_for_c_ref(&input, vtable);
        output.extend(c_ref_impl);
    }

    output
}

fn impl_cdrop_for_vtable(vtable: &ItemStruct) -> TokenStream {
    let name = &vtable.ident;
    let generics = &vtable.generics;

    quote! {
        impl #generics CDrop for #name #generics {
            fn drop(repr: CRefMut<Self>) {
                unsafe { (repr.get_vtable().drop)(repr) }
            }
        }
    }
}

fn create_vtable(input: &ItemTrait) -> Result<ItemStruct, syn::Error> {
    let trait_ident = &input.ident;
    let repr_c: Attribute = parse_quote! {
       #[repr(C)]
    };
    let string_ident = trait_ident.to_string();
    let docs: Attribute = parse_quote! {
        #[doc = concat!("A repr C vtable for the trait ", #string_ident)]
    };
    let mut vtable = ItemStruct {
        attrs: vec![repr_c, docs],
        vis: input.vis.clone(),
        struct_token: syn::token::Struct {
            span: proc_macro2::Span::call_site(),
        },
        ident: syn::Ident::new(
            &format!("{}VTable", input.ident),
            proc_macro2::Span::call_site(),
        ),
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
                colon_token: None,
                ty: syn::Type::BareFn(ty),
            }
        })
        .collect::<Vec<_>>();

    let drop_field: Field = parse_quote!(
        /// A function pointer to the drop function for the trait
        pub drop: unsafe extern "C" fn(CRefMut<Self>)
    );
    fields.push(drop_field);

    vtable.fields = syn::Fields::Named(syn::FieldsNamed {
        brace_token: syn::token::Brace::default(),
        named: fields.into_iter().collect(),
    });

    Ok(vtable)
}

fn impl_vtable_methods(input: &ItemTrait, vtable: &ItemStruct) -> TokenStream {
    let trait_ident = &input.ident;
    let vtable_ident = &vtable.ident;

    let methods = input
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
            let output = &f.sig.output;
            let inputs = map_inputs(&f.sig.inputs, Some(vtable_ident.clone()))
                .enumerate()
                .map(|(i, mut arg)| {
                    let name = format_ident!("arg_{}", i);
                    arg.name = Some((name, Default::default()));
                    arg
                })
                .collect::<Vec<_>>();

            let pass_in_args = inputs
                .clone()
                .into_iter()
                .map(|arg| {
                    let name = arg.name.unwrap().0;

                    if let syn::Type::Path(path) = &arg.ty {
                        let path = path.path.segments.first().map(|s| s.ident.to_string());
                        match path.as_ref().map(|s| s.as_str()) {
                            Some("CRef") => {
                                quote! {
                                    &*(#name.as_ptr() as *const T)
                                }
                            }
                            Some("CRefMut") => {
                                quote! {
                                    &mut *(#name.as_ptr() as *mut T)
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

            quote! {
                #method_name: {
                    unsafe extern "C" fn #method_name<T: #trait_ident>(
                        #(#inputs),*
                    ) #output {
                        #[allow(unsafe_code)]
                        unsafe {
                            T::#method_name(
                                #(#pass_in_args),*
                            )
                        }
                    }
                    #method_name::<T>
                }
            }
        });

    let vtable_creater = quote! {
    #vtable_ident {
         #(#methods),*,

       drop: {
           unsafe extern "C" fn drop<T: #trait_ident>(arg_0: CRefMut<#vtable_ident>) {
               #[allow(unsafe_code)]
               unsafe {
                   ::core::mem::drop(Box::from_raw(arg_0.as_ptr() as *mut T));
               }
           }
           drop::<T>
       },
      }};

    quote! {
        impl #vtable_ident {
            /// Creates a new vtable for the type T that implements the trait
            pub fn new_boxed<T: #trait_ident + 'static>(input: T) -> CRepr<#vtable_ident> {
                let vtable  = #vtable_ident::create_vtable::<T>();
                CRepr::new_boxed(vtable, input)
            }

            /// Creates a new vtable for the type T then store in a static variable in a hea
            pub fn create_vtable<T: #trait_ident + 'static>() -> &'static #vtable_ident {
                   static FN_MAP: std::sync::LazyLock<std::sync::Mutex<std::collections::HashMap<std::any::TypeId, &'static #vtable_ident>>> =
                        std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

                    let type_id = std::any::TypeId::of::<T>();

                    let mut map = FN_MAP.lock().unwrap();
                    let entry = map.entry(type_id).or_insert_with(|| {
                        let vtable = Box::new(#vtable_creater);
                        Box::leak(vtable)
                    });
                    &entry


            }
        }
    }
}

fn map_inputs(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, Token![,]>,
    name: Option<syn::Ident>,
) -> impl Iterator<Item = BareFnArg> {
    let name = name.unwrap_or(format_ident!("Self"));
    inputs.iter().map(move |arg| match arg {
        syn::FnArg::Receiver(recv) => {
            let ty = recv.ty.as_ref().clone();

            match ty {
                syn::Type::Reference(type_ref) if type_ref.mutability.is_none() => BareFnArg {
                    attrs: recv.attrs.clone(),
                    name: None,
                    ty: parse_quote!(CRef<#name>),
                },
                syn::Type::Reference(_) => BareFnArg {
                    attrs: recv.attrs.clone(),
                    name: None,
                    ty: parse_quote!(CRefMut<#name>),
                },
                syn::Type::Path(_) => BareFnArg {
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

fn impl_trait_for_c_ref(input: &ItemTrait, vtable: &ItemStruct) -> TokenStream {
    let trait_ident = &input.ident;
    let vtable_ident = &vtable.ident;
    let methods = input
        .items
        .clone()
        .into_iter()
        .filter_map(|i| {
            if let TraitItem::Fn(fn_item) = i {
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
                        syn::Type::Reference(type_ref) if type_ref.mutability.is_none() => {
                            quote! {
                                self.as_cref()
                            }
                        }
                        syn::Type::Reference(_) => {
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

    quote! {
        impl #trait_ident for CRepr<#vtable_ident> {
            #(#methods)*
        }
    }
}
