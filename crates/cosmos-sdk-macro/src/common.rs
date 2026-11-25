use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Expr, ExprLit, ExprTuple, Fields, Generics, Lit, LitStr, Token,
    Type, ext::IdentExt, parse_quote, punctuated::Punctuated,
};

pub struct Member {
    pub key: LitStr,
    pub ident: syn::Ident,
    pub ty: Type,
    pub flags: Vec<(LitStr, LitStr)>,
    pub base: bool,
}

pub fn members(
    input: &DeriveInput,
    allow_base: bool,
    reserve_name: bool,
) -> syn::Result<Vec<Member>> {
    let Data::Struct(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "only structs are supported",
        ));
    };

    let Fields::Named(fields) = &data.fields else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "only structs with named fields are supported",
        ));
    };

    fields
        .named
        .iter()
        .map(|field| {
            let ident = field
                .ident
                .clone()
                .ok_or_else(|| syn::Error::new_spanned(field, "expected a named field"))?;

            let mut base = false;
            let mut flags = Vec::new();

            for attr in &field.attrs {
                if attr.path().is_ident("base") {
                    if !allow_base {
                        return Err(syn::Error::new_spanned(
                            attr,
                            "`#[base]` is only supported on entry fields",
                        ));
                    }

                    attr.meta.require_path_only()?;
                    base = true;
                } else if attr.path().is_ident("flags") {
                    flags.extend(parse_flags(attr)?);
                }
            }

            if base && !flags.is_empty() {
                return Err(syn::Error::new_spanned(
                    field,
                    "`#[base]` and `#[flags]` cannot be combined",
                ));
            }

            let key = ident.unraw().to_string();

            if reserve_name && !base && key == "name" {
                return Err(syn::Error::new_spanned(
                    &field.ty,
                    "`name` is built into every entry",
                ));
            }

            Ok(Member {
                key: LitStr::new(&key, ident.span()),
                ident,
                ty: field.ty.clone(),
                flags,
                base,
            })
        })
        .collect()
}

pub fn bounded(generics: &Generics, members: &[Member]) -> Generics {
    let mut generics = generics.clone();
    let predicates = &mut generics.make_where_clause().predicates;

    for Member { ty, base, .. } in members {
        predicates.push(if *base {
            parse_quote!(#ty: ::cosmos_sdk::Entry)
        } else {
            parse_quote!(#ty: ::cosmos_sdk::Field)
        });
    }

    generics
}

pub fn define(member: &Member) -> TokenStream {
    let Member {
        key,
        ty,
        flags,
        base,
        ..
    } = member;

    if *base {
        return quote!(<#ty as ::cosmos_sdk::Entry>::fields(out););
    }

    let flags = flags.iter().map(|(key, value)| quote!((#key, #value)));

    quote!(::cosmos_sdk::__private::define::<#ty>(#key, &[#(#flags),*], out);)
}

pub fn read(member: &Member) -> TokenStream {
    let Member { key, ty, base, .. } = member;

    if *base {
        quote!(<#ty as ::cosmos_sdk::Entry>::read(node)?)
    } else {
        quote!(::cosmos_sdk::__private::member::<#ty>(node, #key)?)
    }
}

fn parse_flags(attr: &Attribute) -> syn::Result<Vec<(LitStr, LitStr)>> {
    attr.parse_args_with(Punctuated::<ExprTuple, Token![,]>::parse_terminated)?
        .iter()
        .map(parse_pair)
        .collect()
}

fn parse_pair(tuple: &ExprTuple) -> syn::Result<(LitStr, LitStr)> {
    let text = |expr: &Expr| match expr {
        Expr::Lit(ExprLit {
                      lit: Lit::Str(value),
                      ..
                  }) => Ok(value.clone()),
        other => Err(syn::Error::new_spanned(other, "expected a string literal")),
    };

    match tuple.elems.iter().collect::<Vec<_>>()[..] {
        [key, value] => Ok((text(key)?, text(value)?)),
        _ => Err(syn::Error::new_spanned(
            tuple,
            "expected a `(\"key\", \"value\")` pair",
        )),
    }
}