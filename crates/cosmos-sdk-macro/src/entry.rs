use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    DeriveInput, Generics, LitInt, LitStr, Token, parse_quote,
    punctuated::Punctuated,
};

use crate::common::{self, Member};

#[derive(Default)]
struct Args {
    kind: Option<LitStr>,
    icon: Option<LitStr>,
    color: Option<LitStr>,
    version: Option<LitInt>,
}

pub(super) fn attribute(
    args: TokenStream,
    input: TokenStream,
) -> syn::Result<TokenStream> {
    let mut input: DeriveInput = syn::parse2(input)?;

    input
        .attrs
        .push(parse_quote!(#[derive(::cosmos_sdk::Entry)]));

    if !args.is_empty() {
        input.attrs.push(parse_quote!(#[entry_meta(#args)]));
    }

    Ok(quote! { #input })
}

pub(super) fn expand(input: DeriveInput) -> syn::Result<TokenStream> {
    let args = parse_args(&input)?;
    let members = common::members(&input, true, true)?;

    let own_tags: Vec<LitStr> = input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("tags_internal"))
        .map(|attr| {
            attr.parse_args_with(
                Punctuated::<LitStr, Token![,]>::parse_terminated,
            )
        })
        .collect::<syn::Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect();

    let mut generics = input.generics.clone();

    for member in &members {
        common::add_bound(
            &mut generics,
            &member.ty,
            if member.base {
                parse_quote!(::cosmos_sdk::Entry)
            } else {
                parse_quote!(::cosmos_sdk::Field)
            },
        );
    }

    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) =
        generics.split_for_impl();

    let base_tags: Vec<_> = members
        .iter()
        .filter(|member| member.base)
        .map(|member| {
            let ty = &member.ty;

            quote! {
                for tag in <#ty as ::cosmos_sdk::Taggable>::tags() {
                    if !out.contains(&tag) {
                        out.push(tag);
                    }
                }
            }
        })
        .collect();

    let definitions: Vec<_> =
        members.iter().map(define_member).collect();

    let locals: Vec<_> = (0..members.len())
        .map(|index| format_ident!("__member{index}"))
        .collect();

    let reads: Vec<_> = members
        .iter()
        .zip(&locals)
        .map(|(member, local)| read_member(member, local))
        .collect();

    let names = members.iter().map(|member| &member.ident);
    let own_tags = own_tags.iter();

    let registered = registration(&args, ident, &generics);

    Ok(quote! {
        impl #impl_generics ::cosmos_sdk::Taggable
            for #ident #type_generics #where_clause
        {
            fn tags() -> ::std::vec::Vec<::std::string::String> {
                let mut out = ::std::vec::Vec::new();

                #(#base_tags)*

                for tag in [#(#own_tags),*] {
                    let tag = tag.to_string();

                    if !out.contains(&tag) {
                        out.push(tag);
                    }
                }

                out
            }
        }

        impl #impl_generics ::cosmos_sdk::Entry
            for #ident #type_generics #where_clause
        {
            fn fields(
                out: &mut ::std::vec::Vec<
                    ::cosmos_sdk::computer_model::Field
                >,
            ) {
                #(#definitions)*
            }

            fn read(
                node: ::cosmos_sdk::Node<'_>,
            ) -> ::std::result::Result<Self, ::std::string::String> {
                #(#reads)*

                ::std::result::Result::Ok(Self {
                    #(#names: #locals),*
                })
            }
        }

        impl #impl_generics ::std::convert::TryFrom<
            &::cosmos_sdk::computer_model::Entry
        > for #ident #type_generics #where_clause {
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

fn define_member(member: &Member) -> TokenStream {
    let ty = &member.ty;

    if member.base {
        return quote! {
            <#ty as ::cosmos_sdk::Entry>::fields(out);
        };
    }

    let key = LitStr::new(&member.key, member.ident.span());

    let flags = member.flags.iter().map(|(key, value)| {
        quote! { (#key, #value) }
    });

    quote! {
        ::cosmos_sdk::__private::define::<#ty>(
            #key,
            &[#(#flags),*],
            out
        );
    }
}

fn read_member(member: &Member, local: &syn::Ident) -> TokenStream {
    let ty = &member.ty;

    if member.base {
        quote! {
            let #local = <#ty as ::cosmos_sdk::Entry>::read(node)?;
        }
    } else {
        let key = LitStr::new(&member.key, member.ident.span());

        quote! {
            let #local = ::cosmos_sdk::__private::member::<#ty>(
                node,
                #key
            )?;
        }
    }
}

fn registration(
    args: &Args,
    ident: &syn::Ident,
    generics: &Generics,
) -> TokenStream {
    let Some(kind) = &args.kind else {
        return TokenStream::new();
    };

    let icon = args
        .icon
        .clone()
        .unwrap_or_else(|| LitStr::new("", kind.span()));

    let color = args
        .color
        .clone()
        .unwrap_or_else(|| LitStr::new("", kind.span()));

    let version = args
        .version
        .clone()
        .unwrap_or_else(|| LitInt::new("1", kind.span()));

    let (impl_generics, type_generics, where_clause) =
        generics.split_for_impl();

    quote! {
        impl #impl_generics ::cosmos_sdk::RegisteredEntry
            for #ident #type_generics #where_clause
        {
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
            if meta.path.is_ident("kind") {
                args.kind = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("icon") {
                args.icon = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("color") {
                args.color = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("version") {
                args.version = Some(meta.value()?.parse()?);
            } else {
                return Err(meta.error(
                    "unknown argument; expected `kind`, `icon`, `color` or `version`",
                ));
            }

            Ok(())
        })?;
    }

    if args.kind.is_none()
        && (args.icon.is_some()
        || args.color.is_some()
        || args.version.is_some())
    {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "`icon`, `color` and `version` require a `kind`",
        ));
    }

    Ok(args)
}