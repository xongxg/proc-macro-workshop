use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub(crate) fn default_new(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;

    let expanded = quote! {
        impl #name {
            pub fn new() -> Self {
                Default::default()
            }
        }
    };

    TokenStream::from(expanded)
}
