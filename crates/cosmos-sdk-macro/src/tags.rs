use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Item, LitStr, Token,
    punctuated::Punctuated,
    parse::Parser,
};

pub(super) fn expand(
    args: TokenStream,
    input: TokenStream,
) -> syn::Result<TokenStream> {
    let tags: Vec<LitStr> =
        Punctuated::<LitStr, Token![,]>::parse_terminated
            .parse2(args)?
            .into_iter()
            .collect();

    let mut item: Item = syn::parse2(input)?;

    match &mut item {
        Item::Struct(item) => {
            let tags = &tags;

            item.attrs.push(syn::parse_quote! {
                #[tags_internal(#(#tags),*)]
            });
        }

        Item::Trait(item) => {
            if item.items.iter().any(|item| {
                matches!(
                    item,
                    syn::TraitItem::Fn(method)
                        if method.sig.ident == "__cosmos_sdk_tags"
                )
            }) {
                return Err(syn::Error::new_spanned(
                    &item.ident,
                    "trait already defines `__cosmos_sdk_tags`",
                ));
            }

            let tags = &tags;

            item.items.push(syn::parse_quote! {
                #[doc(hidden)]
                fn __cosmos_sdk_tags() -> &'static [&'static str]
                where
                    Self: Sized,
                {
                    &[#(#tags),*]
                }
            });
        }

        _ => {
            return Err(syn::Error::new_spanned(
                item,
                "`#[tags(...)]` can only be applied to structs and traits",
            ));
        }
    }

    Ok(quote! { #item })
}

pub(super) fn expand_taggable(
    args: TokenStream,
    input: TokenStream,
) -> syn::Result<TokenStream> {
    if !args.is_empty() {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "`#[taggable]` does not accept arguments",
        ));
    }

    let item: Item = syn::parse2(input)?;

    let Item::Impl(item) = item else {
        return Err(syn::Error::new_spanned(
            item,
            "`#[taggable]` can only be applied to trait implementations",
        ));
    };

    if item.trait_.is_none() {
        return Err(syn::Error::new_spanned(
            &item,
            "`#[taggable]` requires a trait implementation",
        ));
    }

    Ok(quote! { #item })
}