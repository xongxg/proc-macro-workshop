use quote::quote;
use syn::parse::Parser;
use syn::{Error, Expr, ExprLit, Lit, LitStr};

pub fn do_expand(st: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let ret = generate_debug_trait(st)?;
    Ok(ret)
}

type StructFields = syn::punctuated::Punctuated<syn::Field, syn::Token![,]>;
fn get_fields_from_derive_input(d: &syn::DeriveInput) -> syn::Result<&StructFields> {
    if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = d.data
    {
        return Ok(named);
    }
    Err(syn::Error::new_spanned(
        d,
        "Must define on a Struct, not Enum".to_string(),
    ))
}

fn get_custom_format_fields(field: &syn::Field) -> syn::Result<Option<String>> {
    for attr in &field.attrs {
        if attr.meta.path().segments[0].ident.to_string().as_str() == "debug" {
            let val = &attr.meta.require_name_value()?.value;
            if let syn::Expr::Lit(ExprLit {
                lit: Lit::Str(litstr),
                ..
            }) = val
            {
                // eprintln!(
                //     "ident litstr.value ===> {:#?}",
                //     litstr.value()
                // );

                return Ok(Some(litstr.value()));
            }
        }
    }

    Ok(None)
}

fn generate_debug_trait(st: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let fields = get_fields_from_derive_input(st)?;
    let struct_name_ident = &st.ident;
    let struct_name_literal = struct_name_ident.to_string();

    let mut fmt_body_stream = proc_macro2::TokenStream::new();

    fmt_body_stream.extend(quote!(
        fmt.debug_struct(#struct_name_literal) // 注意这里引用的是一个字符串，不是一个syn::Ident，生成的代码会以字面量形式表示出来
    ));
    for field in fields.iter() {
        let field_name_indent = field.ident.as_ref().unwrap();
        let field_name_literal = field_name_indent.to_string();

        let mut format_str = "{:?}".to_string();
        if let Some(format) = get_custom_format_fields(field)? {
            format_str = format;
        }

        fmt_body_stream.extend(quote!(
            // .field(#field_name_literal, &self.#field_name_indent)  // 这行同样注意literal和ident的区别
            .field(#field_name_literal, &format_args!(#format_str,&self.#field_name_indent))
        ));
    }

    fmt_body_stream.extend(quote!(
        .finish()
    ));

    let ret_stream = quote!(
        impl std::fmt::Debug for #struct_name_ident {
            fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
                #fmt_body_stream
            }
        }
    );

    Ok(ret_stream)
}
