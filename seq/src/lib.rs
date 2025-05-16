use proc_macro::TokenStream;

#[proc_macro]
pub fn add(input: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro::TokenTree;

    #[test]
    fn it_works() {
        // TokenTree::Group()
    }
}
