use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{
    Attribute, BareFnArg, Field, Ident, ItemStruct, ItemTrait, LitStr, Token, TraitItem, Type,
    TypeBareFn, TypeParamBound, Visibility, parse_quote, spanned::Spanned,
};

pub fn impl_crusty_trait(input: ItemTrait) -> TokenStream {
    let mut output = input.to_token_stream();

    let super_traits = get_super_traits(&input);

    let vtable = create_vtable(&input, &super_traits);
    output.extend(
        vtable
            .as_ref()
            .map(|vtable| vtable.to_token_stream())
            .unwrap_or_else(|err| err.to_compile_error()),
    );

    if let Ok(vtable) = &vtable {
        impl_as_vtable_for_super_traits(&super_traits, &vtable.ident)
            .for_each(|impl_block| output.extend(impl_block));

        
        let cdrop = impl_cdrop_for_vtable(vtable);
        output.extend(cdrop);
        
        let vtable_methods = impl_vtable_methods(&input, vtable);
        output.extend(vtable_methods);
        let c_ref_impl = impl_trait_for_c_ref(&input, vtable);
        output.extend(c_ref_impl);
        let impl_c_ref = impl_trait_for_c_ref_where_as_vtable(&input, vtable);
        output.extend(impl_c_ref);
    }

    output
}

fn impl_trait_for_c_ref_where_as_vtable(input: &ItemTrait, vtable: &ItemStruct) -> TokenStream {
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
                let methods = self.as_vtable();
                #[allow(unsafe_code)]
                unsafe {
                    (methods.#method_name)(#(#inputs),*)
                }
            }));

            f.to_token_stream()
        });

    quote! {
        impl<T> #trait_ident for CRepr<T>
        where
            T: AsVTable<&'static #vtable_ident> + CDrop,
        {
            #(#methods)*
        }
    }
}

const IGNORE_SUPER_TRAITS: &[&str] = &["Send", "Sync"];

#[allow(dead_code)]
struct SuperTrait {
    ident: Ident,
    vtable_ident: Ident,
    vtable_ty: Type,
    field_ident: Ident,
    path: syn::Path,
}

type SuperTraits = Vec<SuperTrait>;

fn get_super_traits(input: &ItemTrait) -> SuperTraits {
    input
        .supertraits
        .iter()
        .filter_map(|super_trait| {
            if let TypeParamBound::Trait(trait_bound) = super_trait {
                let ident = trait_bound.path.get_ident().unwrap();
                if IGNORE_SUPER_TRAITS.contains(&ident.to_string().as_str()) {
                    return None;
                }
                let vtable_ident =
                    Ident::new(&format!("{}VTable", ident), proc_macro2::Span::call_site());
                let field_ident = Ident::new(
                    &format!("super_{}", ident.to_string().to_snake_case()),
                    proc_macro2::Span::call_site(),
                );
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
                            segments: syn::punctuated::Punctuated::from_iter(vec![
                                syn::PathSegment {
                                    ident: vtable_ident.clone(),
                                    arguments: syn::PathArguments::None,
                                },
                            ]),
                        },
                    })),
                });
                Some(SuperTrait {
                    ident: ident.clone(),
                    vtable_ident,
                    field_ident,
                    path: trait_bound.path.clone(),
                    vtable_ty,
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

fn impl_as_vtable_for_super_traits(
    super_traits: &SuperTraits,
    vtable_ident: &Ident,
) -> impl Iterator<Item = TokenStream> {
    super_traits.iter().map(move |super_trait| {
        let field_ident = &super_trait.field_ident;
        let vtable_ty = &super_trait.vtable_ty;

        quote! {
            impl AsVTable<#vtable_ty> for #vtable_ident {
                fn as_vtable(&self) -> #vtable_ty {
                    &self.#field_ident
                }
            }
        }
    })
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

fn create_vtable(input: &ItemTrait, super_traits: &SuperTraits) -> Result<ItemStruct, syn::Error> {
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
        ident: Ident::new(
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
            ident: Some(Ident::new(
                &super_trait.field_ident.to_string(),
                proc_macro2::Span::call_site(),
            )),
            colon_token: None,
            ty: super_trait.vtable_ty.clone(),
        })
        .collect::<Vec<_>>();

    fields.extend(super_trait_fields);

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

                    if let Type::Path(path) = &arg.ty {
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
        })
        .collect::<Vec<_>>();

    let super_trait_field = vtable
        .fields
        .iter()
        .filter_map(|field| {
            if let Type::Reference(ty) = &field.ty {
                if let Type::Path(path) = &*ty.elem {
                    let ident = field.ident.as_ref().unwrap();
                    let super_vtable_ty = path.path.get_ident().unwrap();
                    return Some(quote! {
                        #ident: #super_vtable_ty::create_vtable::<T>()
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
    name: Option<Ident>,
) -> impl Iterator<Item = BareFnArg> {
    let name = name.unwrap_or(format_ident!("Self"));
    inputs.iter().map(move |arg| match arg {
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

    quote! {
        impl #trait_ident for CRepr<#vtable_ident> {
            #(#methods)*
        }
    }
}
