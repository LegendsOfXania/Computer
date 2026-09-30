mod entry;
mod field;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn entry(attr: TokenStream, item: TokenStream) -> TokenStream {
    entry::expand(attr, item)
}

#[proc_macro_derive(Field)]
pub fn field(input: TokenStream) -> TokenStream {
    field::expand(input)
}