use proc_macro::TokenStream;
use quote::quote;
use syn::{DataStruct, DeriveInput};

pub(crate) fn field_counter(input: &DeriveInput) -> TokenStream {
    let ident = &input.ident;
    let count = if let syn::Data::Struct(DataStruct { fields, .. }) = &input.data {
        fields.iter().count()
    } else {
        0
    };

    let expanded = quote! {
        impl #ident {
            pub fn field_count() -> usize {
                #count
            }
        }
    };

    TokenStream::from(expanded)
}


