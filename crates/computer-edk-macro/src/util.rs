use proc_macro2::TokenStream;
use quote::quote;
use syn::{Field, FieldsNamed, FieldsUnnamed};

fn definition(name: String, field: &Field) -> TokenStream {
    let ty = &field.ty;

    quote! {
        <#ty as ::computer_edk::Field>::definition(#name)
    }
}

pub fn named(fields: &FieldsNamed) -> Vec<TokenStream> {
    fields
        .named
        .iter()
        .map(|field| {
            let name = field
                .ident
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default();
            definition(name, field)
        })
        .collect()
}

pub fn unnamed(fields: &FieldsUnnamed) -> Vec<TokenStream> {
    fields
        .unnamed
        .iter()
        .enumerate()
        .map(|(index, field)| definition(index.to_string(), field))
        .collect()
}
