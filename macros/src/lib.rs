#![allow(missing_docs)]

use crusty_trait_macro::impl_crusty_trait;
use quote::ToTokens;
use syn::ItemTrait;

#[proc_macro_attribute]
pub fn crusty_trait(
    _attrs: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as ItemTrait);

    impl_crusty_trait(input).to_token_stream().into()
}
