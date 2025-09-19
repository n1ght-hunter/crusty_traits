use syn::parse_quote;

pub fn impl_cdrop_for_vtable(vtable: &syn::ItemStruct) -> syn::Item {
    let name = &vtable.ident;
    let mut generics = vtable.generics.clone();
    // Clear bounds from generics
    generics.params.iter_mut().for_each(|param| {
        if let syn::GenericParam::Type(type_param) = param {
            type_param.bounds.clear();
        }
    });

    parse_quote! {
        impl #generics CDrop for #name #generics {
            fn drop(repr: CRefMut<Self>) {
                unsafe { (repr.get_vtable().drop)(repr) }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_impl_cdrop_for_vtable() {
        let input: syn::ItemStruct = parse_quote! {
            #[repr(C)]
            pub struct CVecVTable<V> {
                pub push: unsafe extern "C" fn(CRefMut<Self>, V),
                pub extend: unsafe extern "C" fn(CRefMut<Self>, usize),
                pub capacity: unsafe extern "C" fn(CRef<Self>) -> usize,
                pub drop: unsafe extern "C" fn(CRefMut<Self>),
            }
        };
        let output = impl_cdrop_for_vtable(&input);

        let expected: syn::ItemImpl = parse_quote! {
            impl<V> CDrop for CVecVTable<V> {
                fn drop(repr: CRefMut<Self>) {
                    unsafe { (repr.get_vtable().drop)(repr) }
                }
            }
        };

        assert_eq!(output, syn::Item::Impl(expected.clone()));
    }
}
