use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Fields, Ident, ItemStruct, LitInt, LitStr, Path, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

use crate::util;

struct Args {
    kind: LitStr,
    base: Option<Path>,
    version: Option<LitInt>,
    tags: Vec<LitStr>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut kind = None;
        let mut base = None;
        let mut version = None;
        let mut tags: Option<Vec<LitStr>> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;

            match key.to_string().as_str() {
                "kind" => set(&mut kind, &key, || {
                    input.parse::<Token![=]>()?;
                    input.parse()
                })?,
                "base" => set(&mut base, &key, || {
                    input.parse::<Token![=]>()?;
                    input.parse()
                })?,
                "version" => set(&mut version, &key, || {
                    input.parse::<Token![=]>()?;
                    input.parse()
                })?,
                "tags" => set(&mut tags, &key, || {
                    let content;
                    syn::parenthesized!(content in input);
                    Ok(Punctuated::<LitStr, Token![,]>::parse_terminated(&content)?
                        .into_iter()
                        .collect())
                })?,
                _ => {
                    return Err(syn::Error::new_spanned(
                        key,
                        "unknown argument, expected `kind`, `base`, `version` or `tags(...)`",
                    ));
                }
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        let kind: LitStr = kind.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "#[entry] requires `kind = \"...\"`",
            )
        })?;

        let tags = tags.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "#[entry] requires `tags(...)`, even empty: `tags()`",
            )
        })?;

        if kind.value().trim().is_empty() {
            return Err(syn::Error::new_spanned(kind, "`kind` cannot be empty"));
        }

        Ok(Self {
            kind,
            base,
            version,
            tags,
        })
    }
}

fn set<T>(
    slot: &mut Option<T>,
    key: &Ident,
    parse: impl FnOnce() -> syn::Result<T>,
) -> syn::Result<()> {
    if slot.is_some() {
        return Err(syn::Error::new_spanned(key, format!("duplicate `{key}`")));
    }

    *slot = Some(parse()?);

    Ok(())
}

pub fn expand(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as Args);
    let item = parse_macro_input!(item as ItemStruct);

    if !item.generics.params.is_empty() {
        return syn::Error::new_spanned(&item.generics, "#[entry] does not support generics")
            .to_compile_error()
            .into();
    }

    let Fields::Named(fields) = &item.fields else {
        return syn::Error::new_spanned(
            &item.fields,
            "#[entry] requires a struct with named fields",
        )
        .to_compile_error()
        .into();
    };

    let ident = &item.ident;
    let kind = &args.kind;
    let tags = &args.tags;
    let definitions = util::named(fields);

    let version = match &args.version {
        Some(version) => quote!(#version),
        None => quote!(0),
    };

    let inherited = match &args.base {
        Some(base) => quote!(<#base as ::computer_edk::Entry>::definition().fields),
        None => quote!(::std::vec::Vec::new()),
    };

    quote! {
        #item

        impl ::computer_edk::Entry for #ident {
            fn definition() -> ::computer_edk::model::EntryDefinition {
                let mut fields = #inherited;
                fields.extend(::std::vec![#(#definitions),*]);

                ::computer_edk::model::EntryDefinition {
                    kind: ::std::string::String::from(#kind),
                    version: #version,
                    tags: ::std::vec![#(::std::string::String::from(#tags)),*],
                    fields,
                }
            }
        }
    }
    .into()
}
