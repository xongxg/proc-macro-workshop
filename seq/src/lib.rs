// mod seq_impl;
mod seq_impl2;
mod seq_impl;

// use crate::seq_impl::SeqMacroInput;
use syn::parse::Parse;
use syn::parse_macro_input;
use crate::seq_impl2::SeqMacroInput2;

#[proc_macro]
pub fn seq(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // let input = parse_macro_input!(input as SeqMacroInput);

    // let buff = TokenBuffer::new2(input.body.clone());
    // let (res, found) = input.find_block_to_expand_and_do_expand(buff.begin());
    // if found {
    //     return res.into();
    // }
    //
    // /// not match #(xxxxxxxx)*
    // let mut res = proc_macro2::TokenStream::new();
    // for i in input.start..input.end {
    //     res.extend(input.expand(&input.body, i));
    // }

    let input = parse_macro_input!(input as SeqMacroInput2);
    let output: proc_macro2::TokenStream = input.into();
    output.into()
}
