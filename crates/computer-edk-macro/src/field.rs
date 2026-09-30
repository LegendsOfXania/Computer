use proc_macro::TokenStream;

use quote::quote;
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Fields, FieldsNamed, FieldsUnnamed, Variant,
    parse_macro_input,
};

pub fn expand(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ident = &input.ident;

    let body = match &input.data {
        Data::Struct(data) => expand_struct(data),
        Data::Enum(data) => expand_enum(data),
        Data::Union(_) => {
            return syn::Error::new_spanned(
                ident,
                "`Field` cannot be derived for a union",
            )
            .to_compile_error()
            .into();
        }
    };

    let body = match body {
        Ok(body) => body,
        Err(error) => return error.to_compile_error().into(),
    };

    quote! {
        impl ::computer_edk::Field for #ident {
            fn definition(
                name: &str,
            ) -> ::computer_model::field::FieldDefinition {
                ::computer_model::field::FieldDefinition {
                    name: name.into(),
                    kind: #body,
                    flags: Vec::new(),
                }
            }
        }
    }
    .into()
}

fn expand_struct(data: &DataStruct) -> syn::Result<proc_macro2::TokenStream> {
    let fields = match &data.fields {
        Fields::Named(fields) => fields,
        _ => {
            return Err(syn::Error::new_spanned(
                &data.fields,
                "`Field` requires structs to have named fields",
            ));
        }
    };

    let definitions = named_field_definitions(fields);

    Ok(quote! {
        ::computer_model::field::FieldKind::Struct(vec![
            #(#definitions),*
        ])
    })
}

fn expand_enum(data: &DataEnum) -> syn::Result<proc_macro2::TokenStream> {
    if data.variants.is_empty() {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "`Field` requires enums to have at least one variant",
        ));
    }

    let is_simple = data
        .variants
        .iter()
        .all(|variant| matches!(variant.fields, Fields::Unit));

    if is_simple {
        let variants = data.variants.iter().map(|variant| {
            let name = variant.ident.to_string();

            quote! {
                #name.to_string()
            }
        });

        return Ok(quote! {
            ::computer_model::field::FieldKind::Enum(vec![
                #(#variants),*
            ])
        });
    }

    let cases = data
        .variants
        .iter()
        .map(expand_variant)
        .collect::<syn::Result<Vec<_>>>()?;

    Ok(quote! {
        ::computer_model::field::FieldKind::Variant(vec![
            #(#cases),*
        ])
    })
}

fn expand_variant(
    variant: &Variant,
) -> syn::Result<proc_macro2::TokenStream> {
    let name = variant.ident.to_string();

    let fields = match &variant.fields {
        Fields::Unit => {
            quote! {
                Vec::new()
            }
        }

        Fields::Named(fields) => {
            let definitions = named_field_definitions(fields);

            quote! {
                vec![
                    #(#definitions),*
                ]
            }
        }

        Fields::Unnamed(fields) => {
            let definitions = unnamed_field_definitions(fields);

            quote! {
                vec![
                    #(#definitions),*
                ]
            }
        }
    };

    Ok(quote! {
        ::computer_model::field::Case {
            name: #name.into(),
            fields: #fields,
        }
    })
}

fn named_field_definitions(
    fields: &FieldsNamed,
) -> Vec<proc_macro2::TokenStream> {
    fields
        .named
        .iter()
        .map(|field| {
            let name = field
                .ident
                .as_ref()
                .expect("named fields always have an identifier");

            let name = name.to_string();
            let ty = &field.ty;

            quote! {
                <#ty as ::computer_edk::Field>::definition(#name)
            }
        })
        .collect()
}

fn unnamed_field_definitions(
    fields: &FieldsUnnamed,
) -> Vec<proc_macro2::TokenStream> {
    fields
        .unnamed
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let name = index.to_string();
            let ty = &field.ty;

            quote! {
                <#ty as ::computer_edk::Field>::definition(#name)
            }
        })
        .collect()
}