use proc_macro2::TokenTree;
use quote::quote;
use syn::parse::{Parse, ParseStream};

#[derive(Debug)]
pub struct SeqMacroInput {
    name: syn::Ident,
    pub start: usize,
    pub end: usize,
    pub body: proc_macro2::TokenStream,
}

impl Parse for SeqMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<syn::Token![in]>()?;

        let start = input.parse::<syn::LitInt>()?;
        input.parse::<syn::Token![..]>()?;
        let end = input.parse::<syn::LitInt>()?;

        let body_buff;
        syn::braced!(body_buff in input);
        let body = body_buff.parse()?;

        let t = SeqMacroInput {
            name,
            start: start.base10_parse()?,
            end: end.base10_parse()?,
            body,
        };

        Ok(t)
    }
}

impl SeqMacroInput {
    pub fn expand(&self, ts: &proc_macro2::TokenStream, n: usize) -> proc_macro2::TokenStream {
        let buf = ts.clone().into_iter().collect::<Vec<_>>();
        let mut res = proc_macro2::TokenStream::new();

        let mut idx = 0;

        while idx < buf.len() {
            let tree_token = &buf[idx];

            match tree_token {
                TokenTree::Group(g) => {
                    let ch_stream = self.expand(&g.stream(), n);
                    let wait_in_group = proc_macro2::Group::new(g.delimiter(), ch_stream);
                    res.extend(quote::quote! {#wait_in_group});
                }
                TokenTree::Ident(prefix) => {
                    if idx + 2 < buf.len() {
                        if let TokenTree::Punct(p) = &buf[idx + 1] {
                            if p.as_char() == '#' {
                                if let proc_macro2::TokenTree::Ident(i) = &buf[idx + 2] {
                                    if i == &self.name
                                        && prefix.span().end() == p.span().start() // 校验是否连续，无空格
                                        && p.span().end() == i.span().start()
                                    {
                                        let new_ident_litral =
                                            format!("{}{}", prefix.to_string(), n);
                                        let new_ident = proc_macro2::Ident::new(
                                            new_ident_litral.as_str(),
                                            prefix.span(),
                                        );
                                        res.extend(quote::quote!(#new_ident));
                                        idx += 3; // 我们消耗了3个Token，所以这里要加3
                                        continue;
                                    }
                                }
                            }
                        }
                    }

                    if prefix == &self.name {
                        let new_ident = proc_macro2::Literal::usize_unsuffixed(idx);
                        res.extend(quote::quote! {#new_ident});
                        idx += 1;

                        continue;
                    }

                    res.extend(quote! {#tree_token});
                }
                _ => res.extend(quote! {#tree_token}),
            }

            idx += 1;
        }

        res
    }
}
