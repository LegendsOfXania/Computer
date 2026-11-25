use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, ExprLit, Item, Lit, LitStr, Token, parse::Parser, punctuated::Punctuated};

pub(super) fn expand(args: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let mut item: Item = syn::parse2(input)?;

    match &mut item {
        Item::Struct(item) => {
            parse(args.clone())?;

            item.attrs.push(syn::parse_quote!(#[tags_internal(#args)]));
        }

        Item::Trait(item) => {
            let tags = Punctuated::<LitStr, Token![,]>::parse_terminated.parse2(args)?;

            if item.items.iter().any(
                |item| matches!(item, syn::TraitItem::Fn(method) if method.sig.ident == "__cosmos_sdk_tags"),
            ) {
                return Err(syn::Error::new_spanned(
                    &item.ident,
                    "trait already defines `__cosmos_sdk_tags`",
                ));
            }

            item.items.push(syn::parse_quote! {
                #[doc(hidden)]
                fn __cosmos_sdk_tags() -> &'static [&'static str]
                where
                    Self: Sized,
                {
                    &[#tags]
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

    Ok(quote!(#item))
}

pub(super) fn parse(args: TokenStream) -> syn::Result<Vec<TokenStream>> {
    Punctuated::<Expr, Token![,]>::parse_terminated
        .parse2(args)?
        .into_iter()
        .map(|expr| match expr {
            Expr::Lit(ExprLit {
                          lit: Lit::Str(tag), ..
                      }) => Ok(quote!(&[#tag])),
            Expr::Path(path) => Ok(quote!(<Self as #path>::__cosmos_sdk_tags())),
            other => Err(syn::Error::new_spanned(
                other,
                "expected a string literal or a trait",
            )),
        })
        .collect()
}