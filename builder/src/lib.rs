use proc_macro::{Ident, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{DeriveInput, Error, LitStr, Type};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn proc_builder(item: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(item as DeriveInput);
    // eprintln!("{:#?}", ast);

    let name = &ast.ident;
    let b_name = format!("{}Builder", name);
    let bident = syn::Ident::new(&b_name, name.span());

    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = ast.data
    {
        named
    } else {
        unimplemented!()
    };

    let optimized = fields.iter().map(|f| {
        let ty = &f.ty;
        let ident = &f.ident;

        if ty_inner_type("Option", ty).is_some() || build_of(f).is_some() {
            quote! {
                 #ident: #ty,
            }
        } else {
            quote! {
                #ident: std::option::Option <#ty>,
            }
        }
    });

    // for when you call Builder::build
    let build_fields = fields.iter().map(|f| {
        let name = &f.ident;
        if ty_inner_type("Option", &f.ty).is_some() || build_of(f).is_some() {
            quote! {
                #name: self.#name.clone()
            }
        } else {
            quote! {
                #name: self.#name.clone().ok_or(concat!(stringify!(#name), " is not set"))?
            }
        }
    });

    let build_default = fields.iter().map(|f| {
        let ty = &f.ty;
        let ident = &f.ident;

        if build_of(f).is_some() {
            quote! {
                #ident: std::vec::Vec::new()
            }
        } else {
            quote! {
                #ident: None
            }
        }
    });

    let methods = fields.iter().map(|f| {
        let ident = &f.ident;
        let ty = &f.ty;

        let (arg_type, value) = if let Some(inner_ty) = ty_inner_type("Option", ty) {
            // if the field is an Option<T>, setting should take just a T,
            // but we then need to store it within a Some.
            (inner_ty, quote! { std::option::Option::Some(#ident) })
        } else if build_of(&f).is_some() {
            // if the field is a builder, it is a Vec<T>,
            // and the value in the builder is _not_ wrapped in an Option,
            // so we shouldn't wrap the value in Some.
            (ty, quote! { #ident })
        } else {
            // otherwise, we take the type used by the target,
            // and we store it in an Option in the builder
            // in case it was never set.
            (ty, quote! { std::option::Option::Some(#ident) })
        };
        let set_method = quote! {
            pub fn #ident(&mut self, #ident: #arg_type) -> &mut Self {
                self.#ident = #value;
                self
            }
        };

        match extend_method(&f) {
            None => set_method,
            Some((true, extend_method)) => extend_method,
            Some((false, extend_method)) => {
                // safe to generate both!
                let expr = quote! {
                    #set_method
                    #extend_method
                };
                expr.into()
            }
        }
    });

    let doc = format!("\
            Implements the [builder pattern] for [`{}`].\n\
            \n\
            [builder pattern]: https://rust-lang-nursery.github.io/api-guidelines/type-safety.html#c-builder", name);

    
    let expand = quote! {
        #[doc = #doc]
        pub struct #bident {
            #(#optimized)*
        }

        impl #bident {
            #(#methods)*

            pub fn build(&self) -> Result<#name, Box<dyn std::error::Error>> {
                Ok(#name {
                    #(#build_fields),*
                })
            }
        }

        impl #name {
            pub fn builder() -> #bident {
                #bident {
                   #(#build_default),*
                }
            }
        }
    };

    expand.into()
}

fn ty_inner_type<'a>(wrapper: &str, ty: &'a syn::Type) -> Option<&'a syn::Type> {
    if let syn::Type::Path(p) = ty {
        if p.path.segments.len() != 1 || p.path.segments[0].ident != wrapper {
            return None;
        }

        if let syn::PathArguments::AngleBracketed(ref inner_ty) = p.path.segments[0].arguments {
            if inner_ty.args.len() != 1 {
                return None;
            }

            let inner_ty = inner_ty.args.first().unwrap();
            if let syn::GenericArgument::Type(t) = inner_ty {
                return Some(t);
            }
        }
    }
    None
}

fn build_of(f: &syn::Field) -> Option<&syn::Attribute> {
    for attr in &f.attrs {
        if attr.path().segments.len() == 1 && attr.path().segments[0].ident == "builder" {
            return Some(attr);
        }
    }

    None
}

fn extend_method(f: &syn::Field) -> Option<(bool, proc_macro2::TokenStream)> {
    let ident = f.ident.as_ref().unwrap();
    let g = build_of(f);

    if g.is_none() {
        return None;
    }

    // Compile error for the builder attribute
    fn mk_err<T: quote::ToTokens>(t: T) -> Option<(bool, proc_macro2::TokenStream)> {
        Some((
            false,
            syn::Error::new_spanned(t, "expected `builder(each = \"...\")`").to_compile_error(),
        ))
    }

    let mut arg_name = None::<LitStr>;
    let t: Result<(), Error> = g?.parse_nested_meta(|meta| {
        if meta.path.is_ident("each") {
            // eprintln!("meta_path: {:#?}", meta.path);
            let value = meta.value()?;
            let s: LitStr = value.parse()?;
            arg_name = Some(s);
            // eprintln!("meta arg_name: {:#?}", arg_name);

            Ok(())
        } else {
            Err(meta.error("unsupported attribute"))
        }
    });

    if t.is_err() {
        return mk_err(t.err().unwrap().to_compile_error());
    }

    if arg_name == None::<LitStr> {
        return None;
    }

    let arg_name = arg_name?;
    // let arg = syn::Ident::new(&arg_name.value(), arg_name.span());
    let arg = format_ident!("{}", arg_name.value(), span = arg_name.span());
    let inner_ty = ty_inner_type("Vec", &f.ty).unwrap();
    let method = quote! {
        pub fn #arg(&mut self, #arg: #inner_ty) -> &mut Self {
            self.#ident.push(#arg);
            self
        }
    };

    Some((&arg == ident, method))
}
