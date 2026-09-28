use proc_macro::TokenStream;

use quote::quote;
use syn::{
    Ident, ItemStruct, LitStr, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

struct EntryArgs {
    name: LitStr,
    base: Option<Ident>,
}

impl Parse for EntryArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut base = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;

            input.parse::<Token![=]>()?;

            if key == "name" {
                name = Some(input.parse()?);
            } else if key == "base" {
                base = Some(input.parse()?);
            } else {
                return Err(syn::Error::new(
                    key.span(),
                    "unknown #[entry] argument",
                ));
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        let name = name.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "#[entry] requires `name`",
            )
        })?;

        Ok(Self { name, base })
    }
}

#[proc_macro_attribute]
pub fn entry(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as EntryArgs);
    let mut item = parse_macro_input!(item as ItemStruct);

    let name = args.name;
    let base = args.base;

    let ident = &item.ident;

    if let Some(base) = &base {
        if let syn::Fields::Named(fields) = &mut item.fields {
            let field: syn::Field = syn::parse_quote! {
                pub base: #base
            };

            fields.named.insert(0, field);
        } else {
            return syn::Error::new_spanned(
                &item,
                "#[entry(base = ...)] requires a struct with named fields",
            )
            .to_compile_error()
            .into();
        }
    }

    let base_metadata = match &base {
        Some(base) => {
            let base = LitStr::new(&base.to_string(), base.span());

            quote! {
                Some(#base)
            }
        }
        None => {
            quote! {
                None
            }
        }
    };

    quote! {
        #item

        impl #ident {
            pub const ENTRY_NAME: &'static str = #name;
            pub const ENTRY_BASE: Option<&'static str> = #base_metadata;
        }
    }
    .into()
}

struct TagsArgs {
    tags: Punctuated<LitStr, Token![,]>,
}

impl Parse for TagsArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            tags: Punctuated::parse_terminated(input)?,
        })
    }
}

#[proc_macro_attribute]
pub fn tags(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as TagsArgs);
    let item = proc_macro2::TokenStream::from(item);

    let values: Vec<_> = args.tags.into_iter().collect();

    quote! {
        #item

        const _: &[&'static str] = &[
            #(#values),*
        ];
    }
    .into()
}