mod seq_impl;

use crate::seq_impl::SeqMacroInput;
use syn::parse::Parse;
use syn::parse_macro_input;

#[proc_macro]
pub fn seq(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as SeqMacroInput);

    let mut res = proc_macro2::TokenStream::new();
    for i in input.start..input.end {
        res.extend(input.expand(&input.body, i));
    }

    res.into()
}
