use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, parse_quote};

pub fn map_method_ident(ident: Ident) -> Ident {
    Ident::new(&format!("{}METHOD", ident), ident.span())
}

pub fn map_vtable_ident(ident: Ident) -> Ident {
    Ident::new(&format!("{}VTable", ident), ident.span())
}

pub fn map_field_ident(ident: Ident) -> Ident {
    Ident::new(
        &format!("field_{}", ident.to_string().to_snake_case()),
        ident.span(),
    )
}

pub fn repr_c_attribute() -> syn::Attribute {
    parse_quote! {
       #[repr(C)]
    }
}

pub fn doc_attribute(doc: impl Into<String>) -> syn::Attribute {
    let doc: String = doc.into();
    parse_quote! {
       #[doc = #doc]
    }
}

pub fn map_vec_to_generics(vec: &Vec<Ident>) -> TokenStream {
    if vec.is_empty() {
        quote! {}
    } else {
        let generics = vec.iter();
        quote! { < #( #generics ),* > }
    }
}

#[cfg(test)]
pub mod test_utils {

    pub fn item_to_file(item: syn::Item) -> syn::File {
        syn::File {
            shebang: None,
            attrs: vec![],
            items: vec![item],
        }
    }

    pub fn item_to_pretty_string(item: syn::Item) -> String {
        let file = item_to_file(item);
        prettyplease::unparse(&file)
    }
}
