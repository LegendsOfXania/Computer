use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    Fields, Ident, ItemStruct, LitInt, LitStr, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

use crate::base;

struct Args {
    kind: LitStr,
    tags: Vec<LitStr>,
    icon: LitStr,
    color: LitStr,
    version: u32,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut kind: Option<LitStr> = None;
        let mut tags: Option<Vec<LitStr>> = None;
        let mut icon: Option<LitStr> = None;
        let mut color: Option<LitStr> = None;
        let mut version: Option<LitInt> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;

            match key.to_string().as_str() {
                "kind" => set(&mut kind, &key, || {
                    input.parse::<Token![=]>()?;
                    input.parse::<LitStr>()
                })?,
                "tags" => set(&mut tags, &key, || {
                    let content;
                    syn::parenthesized!(content in input);

                    Ok(Punctuated::<LitStr, Token![,]>::parse_terminated(&content)?
                        .into_iter()
                        .collect())
                })?,
                "icon" => set(&mut icon, &key, || {
                    input.parse::<Token![=]>()?;
                    input.parse::<LitStr>()
                })?,
                "color" => set(&mut color, &key, || {
                    input.parse::<Token![=]>()?;
                    input.parse::<LitStr>()
                })?,
                "version" => set(&mut version, &key, || {
                    input.parse::<Token![=]>()?;
                    input.parse::<LitInt>()
                })?,
                _ => {
                    return Err(syn::Error::new_spanned(
                        key,
                        "unknown argument; expected `kind`, `tags(...)`, `icon`, `color` or `version`",
                    ));
                }
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        let kind = kind.ok_or_else(|| {
            syn::Error::new(Span::call_site(), "#[entry] requires `kind = \"...\"`")
        })?;

        if kind.value().trim().is_empty() {
            return Err(syn::Error::new_spanned(kind, "`kind` cannot be empty"));
        }

        let version = version
            .map(|value| value.base10_parse::<u32>())
            .transpose()?
            .unwrap_or(0);

        Ok(Self {
            kind,
            tags: tags.unwrap_or_default(),
            icon: icon.unwrap_or_else(|| LitStr::new("", Span::call_site())),
            color: color.unwrap_or_else(|| LitStr::new("", Span::call_site())),
            version,
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
    let mut item = parse_macro_input!(item as ItemStruct);

    match expand_item(args, &mut item) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

fn expand_item(args: Args, item: &mut ItemStruct) -> syn::Result<TokenStream2> {
    if !item.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &item.generics,
            "#[entry] does not support generic structs",
        ));
    }

    let Fields::Named(fields) = &mut item.fields else {
        return Err(syn::Error::new_spanned(
            &item.fields,
            "#[entry] requires a struct with named fields",
        ));
    };

    let base = base::extract(fields)?;
    let base_name = base.as_ref().map(|base| base.name.clone());

    let mut own_definitions = Vec::new();
    let mut own_to = Vec::new();
    let mut own_from = Vec::new();

    for field in &fields.named {
        if base_name
            .as_ref()
            .is_some_and(|name| field.ident.as_ref() == Some(name))
        {
            continue;
        }

        let name = field
            .ident
            .as_ref()
            .ok_or_else(|| syn::Error::new_spanned(field, "entry fields must be named"))?;
        let key = name.to_string();
        let ty = &field.ty;

        own_definitions.push(quote! {
            <#ty as ::computer_edk::Field>::definition(#key)
        });

        own_to.push(quote! {
            if map.insert(
                ::std::string::String::from(#key),
                ::computer_edk::Field::to_value(&self.#name)?,
            ).is_some() {
                return Err(format!("duplicate field `{}`", #key));
            }
        });

        own_from.push(quote! {
            #name: <#ty as ::computer_edk::Field>::from_value(
                map.shift_remove(#key)
                    .ok_or_else(|| format!("missing field `{}`", #key))?
            )?
        });
    }

    let definition_prefix = match &base {
        Some(base) => {
            let ty = &base.ty;

            quote! {
                let base_definition = <#ty as ::computer_edk::Entry>::definition();
                let mut fields = base_definition.fields;
                let mut tags = base_definition.tags;
            }
        }
        None => quote! {
            let mut fields = ::std::vec::Vec::new();
            let mut tags = ::std::vec::Vec::new();
        },
    };

    let base_to = match &base {
        Some(base) => {
            let name = &base.name;
            let ty = &base.ty;

            quote! {
                let mut map = <#ty as ::computer_edk::Entry>::to_fields(&self.#name)?;
            }
        }
        None => quote! {
            let mut map = ::computer_edk::__private::IndexMap::new();
        },
    };

    let base_from = match &base {
        Some(base) => {
            let name = &base.name;
            let ty = &base.ty;

            quote! {
                let #name = {
                    let base_definition = <#ty as ::computer_edk::Entry>::definition();
                    let mut base_fields =
                        ::computer_edk::__private::IndexMap::with_capacity(
                            base_definition.fields.len()
                        );

                    for definition in base_definition.fields {
                        let field_name = definition.name;
                        let value = map.shift_remove(&field_name).ok_or_else(|| {
                            format!("missing base field `{}`", field_name)
                        })?;

                        base_fields.insert(field_name, value);
                    }

                    <#ty as ::computer_edk::Entry>::from_fields(base_fields)?
                };
            }
        }
        None => quote! {},
    };

    let base_initializer = base
        .as_ref()
        .map(|base| {
            let name = &base.name;
            quote!(#name,)
        })
        .unwrap_or_default();

    let ident = &item.ident;
    let kind = &args.kind;
    let icon = &args.icon;
    let color = &args.color;
    let version = args.version;
    let tags = &args.tags;

    Ok(quote! {
        #item

        impl ::computer_edk::Entry for #ident {
            const KIND: &'static str = #kind;

            fn definition() -> ::computer_edk::model::EntryDefinition {
                #definition_prefix

                fields.extend(::std::vec![#(#own_definitions),*]);

                for tag in ::std::vec![
                    #(::std::string::String::from(#tags)),*
                ] {
                    if !tags.contains(&tag) {
                        tags.push(tag);
                    }
                }

                ::computer_edk::model::EntryDefinition {
                    kind: ::std::string::String::from(#kind),
                    tags,
                    fields,
                    icon: ::std::string::String::from(#icon),
                    color: ::std::string::String::from(#color),
                    version: #version,
                }
            }

            fn to_fields(
                &self,
            ) -> Result<
                ::computer_edk::__private::IndexMap<
                    ::std::string::String,
                    ::computer_edk::model::value::Value
                >,
                ::std::string::String
            > {
                #base_to
                #(#own_to)*

                Ok(map)
            }

            fn from_fields(
                mut map: ::computer_edk::__private::IndexMap<
                    ::std::string::String,
                    ::computer_edk::model::value::Value
                >,
            ) -> Result<Self, ::std::string::String> {
                #base_from

                let value = Self {
                    #(#own_from,)*
                    #base_initializer
                };

                if let Some(name) = map.keys().next() {
                    return Err(format!("unexpected field `{name}`"));
                }

                Ok(value)
            }
        }
    })
}
