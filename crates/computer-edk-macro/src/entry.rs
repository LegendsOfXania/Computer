use proc_macro::TokenStream;

use quote::quote;
use syn::{
    Fields, Ident, ItemStruct, LitStr, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

struct EntryArgs {
    kind: LitStr,
    base: Option<Ident>,
    tags: Vec<LitStr>,
}

impl Parse for EntryArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut kind = None;
        let mut base = None;
        let mut tags = Vec::new();

        while !input.is_empty() {
            let key: Ident = input.parse()?;

            match key.to_string().as_str() {
                "kind" => {
                    input.parse::<Token![=]>()?;

                    if kind.is_some() {
                        return Err(syn::Error::new_spanned(
                            key,
                            "duplicate `kind`",
                        ));
                    }

                    kind = Some(input.parse()?);
                }

                "base" => {
                    input.parse::<Token![=]>()?;

                    if base.is_some() {
                        return Err(syn::Error::new_spanned(
                            key,
                            "duplicate `base`",
                        ));
                    }

                    base = Some(input.parse()?);
                }

                "tags" => {
                    let content;
                    syn::parenthesized!(content in input);

                    let values =
                        Punctuated::<LitStr, Token![,]>::parse_terminated(
                            &content,
                        )?;

                    tags.extend(values);
                }

                _ => {
                    return Err(syn::Error::new_spanned(
                        key,
                        "unknown #[entry] argument",
                    ));
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        let kind = kind.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "#[entry] requires `kind`",
            )
        })?;

        Ok(Self {
            kind,
            base,
            tags,
        })
    }
}

pub fn expand(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as EntryArgs);
    let item = parse_macro_input!(item as ItemStruct);

    let ident = &item.ident;
    let kind = args.kind;
    let base = args.base;
    let tags = args.tags;

    let fields = match &item.fields {
        Fields::Named(fields) => fields,

        _ => {
            return syn::Error::new_spanned(
                &item.fields,
                "#[entry] requires a struct with named fields",
            )
            .to_compile_error()
            .into();
        }
    };

    let field_definitions = fields
        .named
        .iter()
        .map(|field| {
            let name = match &field.ident {
                Some(name) => name.to_string(),

                None => {
                    return syn::Error::new_spanned(
                        field,
                        "#[entry] requires named fields",
                    )
                    .to_compile_error();
                }
            };

            let ty = &field.ty;

            quote! {
                <#ty as ::computer_edk::Field>::definition(#name)
            }
        })
        .collect::<Vec<_>>();

    let base_metadata = match &base {
        Some(base) => {
            quote! {
                Some(#base::ENTRY_KIND)
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
            pub const ENTRY_KIND: &'static str = #kind;

            pub const ENTRY_BASE: Option<&'static str> = #base_metadata;

            pub const TAGS: &'static [&'static str] = &[
                #(#tags),*
            ];

            pub fn fields() -> Vec<::computer_model::field::FieldDefinition> {
                vec![
                    #(#field_definitions),*
                ]
            }

            pub fn blueprint() -> ::computer_edk::EntryBlueprint {
                ::computer_edk::EntryBlueprint {
                    kind: Self::ENTRY_KIND,
                    base: Self::ENTRY_BASE,
                    fields: Self::fields(),
                    tags: Self::TAGS,
                }
            }
        }
    }
    .into()
}