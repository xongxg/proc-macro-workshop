use proc_macro2::token_stream::IntoIter;
use proc_macro2::{Delimiter, TokenStream, TokenTree};
use std::iter;
use syn::parse::ParseStream;
use syn::{braced, LitInt, Token};

#[derive(Debug)]
pub struct SeqMacroInput2 {
    from: syn::LitInt,
    to: syn::LitInt,
    inclusive: bool,
    ident: syn::Ident,
    tt: proc_macro2::TokenStream,
}

impl syn::parse::Parse for SeqMacroInput2 {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<syn::Ident>()?;
        let _in = input.parse::<Token![in]>()?;
        let from = input.parse::<syn::LitInt>()?;
        let inclusive = input.peek(Token![..=]);
        if inclusive {
            <Token![..=]>::parse(input)?;
        } else {
            <Token![..]>::parse(input)?;
        }

        let to = input.parse::<LitInt>()?;
        let content;
        let _braces = braced!(content in input);
        let tt = proc_macro2::TokenStream::parse(&content)?;

        Ok(SeqMacroInput2 {
            from,
            to,
            inclusive,
            tt,
            ident,
        })
    }
}

impl Into<proc_macro2::TokenStream> for SeqMacroInput2 {
    fn into(self) -> TokenStream {
        self.expand(self.tt.clone())
    }
}

#[derive(Debug, Copy, Clone)]
enum Mode {
    ReplaceIdent(usize),
    ReplaceSequence,
}

impl SeqMacroInput2 {
    fn range(&self) -> impl Iterator<Item = usize> {
        if self.inclusive {
            self.from.base10_parse::<usize>().unwrap()..self.to.base10_parse::<usize>().unwrap() + 1
        } else {
            self.from.base10_parse::<usize>().unwrap()..self.to.base10_parse::<usize>().unwrap()
        }
    }

    fn expand_2(
        &self,
        tt: TokenTree,
        token_iter: &mut IntoIter,
        mutated: &mut bool,
        mode: Mode,
    ) -> TokenStream {
        let tt = match tt {
            TokenTree::Group(group) => {
                let (expanded, g_mutated) = self.expand_pass(group.stream(), mode);
                let mut expanded = proc_macro2::Group::new(group.delimiter(), expanded);
                *mutated |= g_mutated;
                expanded.set_span(group.span());
                TokenTree::Group(expanded)
            }
            TokenTree::Ident(ident) if ident == self.ident => {
                if let Mode::ReplaceIdent(i) = mode {
                    let mut lit = proc_macro2::Literal::usize_unsuffixed(i);
                    lit.set_span(ident.span());
                    *mutated = true;
                    TokenTree::Literal(lit)
                } else {
                    TokenTree::Ident(ident.clone())
                }
            }
            TokenTree::Ident(mut prefix) => match (mode, peek_next_two(token_iter)) {
                (
                    Mode::ReplaceIdent(i),
                    (Some(TokenTree::Punct(punct)), Some(TokenTree::Ident(ident2))),
                ) if punct.as_char() == '#' && ident2 == self.ident => {
                    prefix = proc_macro2::Ident::new(&format!("{}{}", prefix, i), prefix.span());
                    *mutated = true;

                    token_iter.next();
                    token_iter.next();

                    TokenTree::Ident(prefix)
                }
                _ => TokenTree::Ident(prefix),
            },
            TokenTree::Punct(punct) if punct.as_char() == '#' => {
                if let Mode::ReplaceIdent(i) = mode {
                    match peek_next_two(token_iter) {
                        (Some(TokenTree::Group(group)), Some(TokenTree::Punct(punct)))
                            if group.delimiter() == Delimiter::Parenthesis
                                && punct.as_char() == '*' =>
                        {
                            *mutated = true;
                            token_iter.next();
                            token_iter.next();

                            return self
                                .range()
                                .map(|i| self.expand_pass(group.stream(), Mode::ReplaceIdent(i)))
                                .map(|(ts, _)| ts)
                                .collect::<TokenStream>();
                        }
                        _ => {}
                    }
                }

                TokenTree::Punct(punct.clone())
            }
            _ => tt,
        };

        iter::once(tt).collect::<TokenStream>()
    }

    fn expand_pass(
        &self,
        body: proc_macro2::TokenStream,
        mode: Mode,
    ) -> (proc_macro2::TokenStream, bool) {
        let mut out = TokenStream::new();
        let mut mutated = false;
        let mut token_iter = body.into_iter();
        while let Some(token) = token_iter.next() {
            out.extend(self.expand_2(token, &mut token_iter, &mut mutated, mode));
        }

        (out, mutated)
    }

    pub fn expand(&self, body: proc_macro2::TokenStream) -> TokenStream {
        let (out, mutated) = self.expand_pass(body.clone(), Mode::ReplaceSequence);
        if mutated {
            return out;
        }

        self.range()
            .map(|i| self.expand_pass(body.clone(), Mode::ReplaceIdent(i)))
            .map(|(ts, _)| ts)
            .collect::<TokenStream>()
    }
}

fn peek_next_two(token_iter: &mut IntoIter) -> (Option<TokenTree>, Option<TokenTree>) {
    let mut token_iter = token_iter.clone();

    (token_iter.next(), token_iter.next())
}
