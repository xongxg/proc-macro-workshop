use quote::quote;
use std::collections::{HashMap, HashSet};
use syn::Pat::Type;
use syn::parse::Parser;
use syn::visit::Visit;
use syn::{ExprLit, Lit, parse_quote, visit};

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
    let fmt_body_stream = generate_debug_trait_core(st)?;

    let fields = get_fields_from_derive_input(st)?;
    let mut field_type_names = Vec::new();
    let mut phantomdata_type_param_names = Vec::new();
    for field in fields {
        if let Some(t) = get_field_type_name(field)? {
            field_type_names.push(t);
        }

        if let Some(t) = get_phantomdata_generic_type_name(field)? {
            phantomdata_type_param_names.push(t);
        }
    }

    let struct_name = &st.ident;
    let mut generics_param_to_modify = st.generics.clone();
    let associated_types_map = get_generic_associated_types(st);
    for mut g in generics_param_to_modify.params.iter_mut() {
        if let syn::GenericParam::Type(t) = g {
            let type_param_name = t.ident.to_string();

            if phantomdata_type_param_names.contains(&type_param_name)
                && !field_type_names.contains(&type_param_name)
            {
                continue;
            }

            if associated_types_map.contains_key(&type_param_name)
                && !field_type_names.contains(&type_param_name)
            {
                continue;
            }

            t.bounds.push(parse_quote!(std::fmt::Debug));
        }
    }

    generics_param_to_modify.make_where_clause();
    for (_, associated_types) in associated_types_map {
        for associated_type in associated_types {
            generics_param_to_modify
                .where_clause
                .as_mut()
                .unwrap()
                .predicates
                .push(parse_quote!(#associated_type:std::fmt::Debug));
        }
    }

    let (impl_generics, type_generics, where_clause) = generics_param_to_modify.split_for_impl();
    let res = quote! {
        impl #impl_generics std::fmt::Debug for #struct_name #type_generics #where_clause {
            fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                #fmt_body_stream
            }
        }
    };
    Ok(res)
}

fn generate_debug_trait_core(input: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let fields = get_fields_from_derive_input(input)?;
    let struct_name_ident = &input.ident;
    let struct_name_literal = struct_name_ident.to_string();
    let mut fmt_body_stream = proc_macro2::TokenStream::new();

    fmt_body_stream.extend(quote!(
        fmt.debug_struct(#struct_name_literal)
    ));
    for field in fields.iter() {
        let field_name_idnet = field.ident.as_ref().unwrap();
        let field_name_literal = field_name_idnet.to_string();

        let mut format_str = "{:?}".to_string();
        if let Some(format) = get_custom_format_fields(field)? {
            format_str = format;
        }
        // 这里是没有指定用户自定义的格式
        fmt_body_stream.extend(quote!(
            .field(#field_name_literal, &format_args!(#format_str, self.#field_name_idnet))
        ));
    }

    fmt_body_stream.extend(quote!(
        .finish()
    ));
    return Ok(fmt_body_stream);
}

fn get_phantomdata_generic_type_name(field: &syn::Field) -> syn::Result<Option<String>> {
    if let syn::Type::Path(syn::TypePath {
        path: syn::Path { ref segments, .. },
        ..
    }) = field.ty
    {
        if let Some(syn::PathSegment { ident, arguments }) = segments.last() {
            if ident == "PhantomData" {
                if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                    args,
                    ..
                }) = arguments
                {
                    if let Some(syn::GenericArgument::Type(syn::Type::Path(gp))) = args.first() {
                        return Ok(Some(gp.path.segments[0].ident.to_string()));
                    }
                }
            }
        }
    }

    Ok(None)
}

fn get_field_type_name(field: &syn::Field) -> syn::Result<Option<String>> {
    if let syn::Type::Path(syn::TypePath {
        path: syn::Path { ref segments, .. },
        ..
    }) = field.ty
    {
        return Ok(Some(segments[0].ident.to_string()));
    }

    Ok(None)
}

struct TypePathVisitor {
    generic_type_names: Vec<String>,
    associated_types: HashMap<String, Vec<syn::TypePath>>,
}

impl<'ast> Visit<'ast> for TypePathVisitor {
    fn visit_type_path(&mut self, path: &'ast syn::TypePath) {
        if path.path.segments.len() >= 2 {
            let generic_type_name = path.path.segments[0].ident.to_string();
            if self.generic_type_names.contains(&generic_type_name) {
                self.associated_types
                    .entry(generic_type_name)
                    .or_insert(Vec::new())
                    .push(path.clone());
            }
        }

        visit::visit_type_path(self, path);
    }
}

fn get_generic_associated_types(input: &syn::DeriveInput) -> HashMap<String, Vec<syn::TypePath>> {
    let origin_generic_param_names: Vec<String> = input
        .generics
        .params
        .iter()
        .filter_map(|f| {
            if let syn::GenericParam::Type(ty) = f {
                return Some(ty.ident.to_string());
            }

            return None;
        })
        .collect();

    let mut visitor = TypePathVisitor {
        generic_type_names: origin_generic_param_names,
        associated_types: HashMap::new(),
    };

    visitor.visit_derive_input(input);
    return visitor.associated_types;
}
