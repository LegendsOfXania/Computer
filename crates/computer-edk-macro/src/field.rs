use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Fields, Variant, parse_macro_input, parse_quote,
};

use crate::util;

pub fn expand(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);

    let kind = match &input.data {
        Data::Struct(data) => expand_struct(data),
        Data::Enum(data) => expand_enum(data),
        Data::Union(_) => Err(syn::Error::new_spanned(
            &input.ident,
            "`Field` cannot be derived for a union",
        )),
    };

    let kind = match kind {
        Ok(kind) => kind,
        Err(error) => return error.to_compile_error().into(),
    };

    let params: Vec<_> = input
        .generics
        .type_params()
        .map(|p| p.ident.clone())
        .collect();
    let predicates = params
        .iter()
        .map(|p| -> syn::WherePredicate { parse_quote!(#p: ::computer_edk::Field) });
    input
        .generics
        .make_where_clause()
        .predicates
        .extend(predicates);

    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        impl #impl_generics ::computer_edk::Field for #ident #ty_generics #where_clause {
            fn definition(name: &str) -> ::computer_edk::model::field::FieldDefinition {
                ::computer_edk::model::field::FieldDefinition {
                    name: name.into(),
                    kind: #kind,
                    flags: ::std::vec::Vec::new(),
                }
            }
        }
    }
    .into()
}

fn expand_struct(data: &DataStruct) -> syn::Result<TokenStream2> {
    let Fields::Named(fields) = &data.fields else {
        return Err(syn::Error::new_spanned(
            &data.fields,
            "`Field` requires a struct with named fields",
        ));
    };

    let definitions = util::named(fields);

    Ok(quote! {
        ::computer_edk::model::field::FieldKind::Struct(::std::vec![#(#definitions),*])
    })
}

fn expand_enum(data: &DataEnum) -> syn::Result<TokenStream2> {
    if data.variants.is_empty() {
        return Err(syn::Error::new_spanned(
            &data.variants,
            "`Field` requires an enum with at least one variant",
        ));
    }

    if data
        .variants
        .iter()
        .all(|variant| matches!(variant.fields, Fields::Unit))
    {
        let names = data
            .variants
            .iter()
            .map(|variant| variant.ident.to_string());

        return Ok(quote! {
            ::computer_edk::model::field::FieldKind::Enum(
                ::std::vec![#(::std::string::String::from(#names)),*]
            )
        });
    }

    let cases = data.variants.iter().map(expand_case);

    Ok(quote! {
        ::computer_edk::model::field::FieldKind::Variant(::std::vec![#(#cases),*])
    })
}

fn expand_case(variant: &Variant) -> TokenStream2 {
    let name = variant.ident.to_string();

    let definitions = match &variant.fields {
        Fields::Unit => Vec::new(),
        Fields::Named(fields) => util::named(fields),
        Fields::Unnamed(fields) => util::unnamed(fields),
    };

    quote! {
        ::computer_edk::model::field::Case {
            name: ::std::string::String::from(#name),
            fields: ::std::vec![#(#definitions),*],
        }
    }
}
