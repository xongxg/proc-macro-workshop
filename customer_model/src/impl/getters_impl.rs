use proc_macro::TokenStream;
use quote::quote;
use syn::{DataStruct, DeriveInput};

pub(crate) fn getters(input: &DeriveInput) -> TokenStream {
    let ident = &input.ident;

    let getters = if let syn::Data::Struct(DataStruct { fields, .. }) = &input.data {
        fields.iter().map(|f| {
            let f_ident = &f.ident;
            let f_ty = &f.ty;

            quote! {
                pub fn #f_ident(&self) -> &#f_ty {
                    &self.#f_ident
                }
            }
        })
    } else {
        unimplemented!()
    };

    let expanded = quote! {
        impl #ident {
            #(#getters)*
        }
    };

    TokenStream::from(expanded)
}
