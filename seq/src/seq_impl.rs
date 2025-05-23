use proc_macro2::{TokenStream, TokenTree};
use quote::quote;
use syn::Token;
use syn::buffer::Cursor;
use syn::parse::{Parse, ParseStream};
use syn::token::Token;



/// Rust过程宏系列教程(4)--实现proc_macro_workshop项目之seq题目
/// https://blog.ideawand.com/2021/10/17/rust_procedural_macro/rust_proc_marco_workshop_guide-04/
///
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
        let inclusive = input.parse::<Token![=]>().ok();
        let end = input.parse::<syn::LitInt>()?.base10_parse()?;

        let body_buff;
        syn::braced!(body_buff in input);
        let body = body_buff.parse()?;

        let t = SeqMacroInput {
            name,
            start: start.base10_parse()?,
            end: if inclusive.is_some() { end + 1 } else { end },
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
                                        let new_ident_litral = format!("{}{}", prefix, n);
                                        let new_ident = proc_macro2::Ident::new(
                                            &new_ident_litral,
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

    pub fn find_block_to_expand_and_do_expand(
        &self,
        cursor: Cursor,
    ) -> (proc_macro2::TokenStream, bool) {
        let mut found = false;
        let mut res = proc_macro2::TokenStream::new();

        let mut cursor = cursor;

        while !cursor.eof() {
            if let Some((punct_prefix, cursor1)) = cursor.punct() {
                if punct_prefix.as_char() == '#' {
                    if let Some((group_prefix, _, cursor2)) =
                        cursor1.group(proc_macro2::Delimiter::Parenthesis)
                    {
                        if let Some((punct_suffix, cursor3)) = cursor2.punct() {
                            if punct_suffix.as_char() == '*' {
                                for i in self.start..self.end {
                                    let t = self.expand(&group_prefix.token_stream(), i);

                                    res.extend(t);
                                }

                                cursor = cursor3;
                                found = true;

                                continue;
                            }
                        }
                    }
                }
            }

            ///#(xxxxx)* not match
            if let Some((group_cur, _, next_cur)) = cursor.group(proc_macro2::Delimiter::Brace) {
                let (t, f) = self.find_block_to_expand_and_do_expand(group_cur);
                res.extend(quote::quote!({#t}));

                cursor = next_cur;
                continue;
            } else if let Some((group_cur, _, next_cur)) =
                cursor.group(proc_macro2::Delimiter::Bracket)
            {
                let (t, f) = self.find_block_to_expand_and_do_expand(group_cur);
                res.extend(quote::quote!({#t}));
                cursor = next_cur;
                continue;
            } else if let Some((group_cur, _, next_cur)) =
                cursor.group(proc_macro2::Delimiter::Parenthesis)
            {
                let (t, f) = self.find_block_to_expand_and_do_expand(group_cur);
                res.extend(quote::quote!({#t}));
                cursor = next_cur;
                continue;
            } else if let Some((punct, next_cur)) = cursor.punct() {
                res.extend(quote::quote!(#punct));
                cursor = next_cur;
                continue;
            } else if let Some((ident, next_cur)) = cursor.ident() {
                res.extend(quote::quote!(#ident));
                cursor = next_cur;
                continue;
            } else if let Some((literal, next_cur)) = cursor.literal() {
                res.extend(quote::quote!(#literal));
                cursor = next_cur;
                continue;
            } else if let Some((lifetime, next_cur)) = cursor.lifetime() {
                // lifetime这种特殊的分类也是用cursor模式来处理的时候特有的，之前`proc_macro2::TokenTree`里面没有定义这个分类
                res.extend(quote::quote!(#lifetime));
                cursor = next_cur;
                continue;
            }
        }

        (res, found)
    }
}
