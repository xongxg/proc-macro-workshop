mod r#impl;

use crate::r#impl::{default_new_impl, field_counter_impl, getters_impl};
use proc_macro::TokenStream;
use syn::token::Token;

#[proc_macro_derive(FieldCounter)]
pub fn field_counter(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    field_counter_impl::field_counter(&input)
}

#[proc_macro_derive(DefaultNew)]
pub fn default_new(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    default_new_impl::default_new(&input)
}

#[proc_macro_derive(Getters)]
pub fn getters(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    getters_impl::getters(&input)
}

