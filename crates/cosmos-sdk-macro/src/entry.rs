use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, LitInt, LitStr, parse_quote};

use crate::{common, tags};

#[derive(Default)]
struct Args {
    kind: Option<LitStr>,
    icon: Option<LitStr>,
    color: Option<LitStr>,
    version: Option<LitInt>,
}

pub(super) fn attribute(args: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let mut input: DeriveInput = syn::parse2(input)?;

    input
        .attrs
        .push(parse_quote!(#[derive(::cosmos_sdk::Entry)]));

    if !args.is_empty() {
        input.attrs.push(parse_quote!(#[entry_meta(#args)]));
    }

    Ok(quote!(#input))
}

pub(super) fn expand(input: DeriveInput) -> syn::Result<TokenStream> {
    let args = parse_args(&input)?;
    let members = common::members(&input, true, true)?;
    let own_tags = own_tags(&input)?;

    let generics = common::bounded(&input.generics, &members);
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let ident = &input.ident;
    let bases = members.iter().filter(|m| m.base).map(|m| &m.ty);
    let defines = members.iter().map(common::define);
    let reads = members.iter().map(common::read);
    let names = members.iter().map(|member| &member.ident);
    let registered = registration(&args, ident, &generics);

    Ok(quote! {
        impl #impl_generics ::cosmos_sdk::Taggable for #ident #type_generics #where_clause {
            fn tags() -> ::std::vec::Vec<::std::string::String> {
                let own: &[&[&str]] = &[#(#own_tags),*];

                let all = ::std::iter::empty::<::std::string::String>()
                    #(.chain(<#bases as ::cosmos_sdk::Taggable>::tags()))*
                    .chain(own.iter().flat_map(|tags| tags.iter()).map(|tag| (*tag).to_owned()));

                let mut out = ::std::vec::Vec::new();

                for tag in all {
                    if !out.contains(&tag) {
                        out.push(tag);
                    }
                }

                out
            }
        }

        impl #impl_generics ::cosmos_sdk::Entry for #ident #type_generics #where_clause {
            fn fields(out: &mut ::std::vec::Vec<::cosmos_sdk::computer_model::Field>) {
                #(#defines)*
            }

            fn read(
                node: ::cosmos_sdk::Node<'_>,
            ) -> ::std::result::Result<Self, ::std::string::String> {
                ::std::result::Result::Ok(Self { #(#names: #reads),* })
            }
        }

        impl #impl_generics ::std::convert::TryFrom<&::cosmos_sdk::computer_model::Entry>
            for #ident #type_generics #where_clause
        {
            type Error = ::std::string::String;

            fn try_from(
                entry: &::cosmos_sdk::computer_model::Entry,
            ) -> ::std::result::Result<Self, Self::Error> {
                <Self as ::cosmos_sdk::Entry>::read(entry.root())
            }
        }

        #registered
    })
}

fn own_tags(input: &DeriveInput) -> syn::Result<Vec<TokenStream>> {
    let mut parts = Vec::new();

    for attr in &input.attrs {
        if attr.path().is_ident("tags_internal") {
            parts.extend(tags::parse(attr.meta.require_list()?.tokens.clone())?);
        }
    }

    Ok(parts)
}

fn registration(args: &Args, ident: &syn::Ident, generics: &syn::Generics) -> TokenStream {
    let Some(kind) = &args.kind else {
        return TokenStream::new();
    };

    let empty = LitStr::new("", kind.span());
    let icon = args.icon.as_ref().unwrap_or(&empty);
    let color = args.color.as_ref().unwrap_or(&empty);
    let version = args
        .version
        .clone()
        .unwrap_or_else(|| LitInt::new("1", kind.span()));

    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics ::cosmos_sdk::RegisteredEntry for #ident #type_generics #where_clause {
            const KIND: &'static str = #kind;
            const ICON: &'static str = #icon;
            const COLOR: &'static str = #color;
            const VERSION: u32 = #version;
        }
    }
}

fn parse_args(input: &DeriveInput) -> syn::Result<Args> {
    let mut args = Args::default();

    for attr in input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("entry_meta"))
    {
        attr.parse_nested_meta(|meta| {
            let slot = if meta.path.is_ident("kind") {
                &mut args.kind
            } else if meta.path.is_ident("icon") {
                &mut args.icon
            } else if meta.path.is_ident("color") {
                &mut args.color
            } else if meta.path.is_ident("version") {
                args.version = Some(meta.value()?.parse()?);
                return Ok(());
            } else {
                return Err(
                    meta.error("unknown argument; expected `kind`, `icon`, `color` or `version`")
                );
            };

            *slot = Some(meta.value()?.parse()?);

            Ok(())
        })?;
    }

    if args.kind.is_none()
        && (args.icon.is_some() || args.color.is_some() || args.version.is_some())
    {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "`icon`, `color` and `version` require a `kind`",
        ));
    }

    Ok(args)
}