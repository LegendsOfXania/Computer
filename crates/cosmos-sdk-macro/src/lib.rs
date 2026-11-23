use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod common;
mod entry;
mod field;

#[proc_macro_attribute]
pub fn entry(args: TokenStream, input: TokenStream) -> TokenStream {
    entry::attribute(args.into(), input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(Entry, attributes(entry_meta, base, flags))]
pub fn derive_entry(input: TokenStream) -> TokenStream {
    entry::expand(parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(Field, attributes(flags))]
pub fn derive_field(input: TokenStream) -> TokenStream {
    field::expand(parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
