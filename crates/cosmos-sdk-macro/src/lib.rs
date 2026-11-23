use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod entry;
mod field;

#[proc_macro_attribute]
pub fn entry(args: TokenStream, input: TokenStream) -> TokenStream {
    entry::attribute(args.into(), input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(Entry, attributes(entry_meta, base, flags))]
pub fn derive_entry(input: TokenStream) -> TokenStream {
    entry::expand(parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(Field, attributes(flags))]
pub fn derive_field(input: TokenStream) -> TokenStream {
    field::expand(parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

mod common {
    use syn::{
        Data, DeriveInput, Expr, ExprLit, ExprTuple, Fields, Generics, Ident, Lit, LitStr, Path,
        Token, Type, ext::IdentExt, parse_quote, punctuated::Punctuated,
    };

    pub struct Member {
        pub key: String,
        pub ident: Ident,
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
                    key,
                    ident,
                    ty: field.ty.clone(),
                    flags,
                    base,
                })
            })
            .collect()
    }

    fn parse_flags(attr: &syn::Attribute) -> syn::Result<Vec<(LitStr, LitStr)>> {
        let pairs = attr.parse_args_with(Punctuated::<ExprTuple, Token![,]>::parse_terminated)?;

        pairs.iter().map(parse_pair).collect()
    }

    fn parse_pair(tuple: &ExprTuple) -> syn::Result<(LitStr, LitStr)> {
        let text = |expr: &Expr| match expr {
            Expr::Lit(ExprLit {
                          lit: Lit::Str(value),
                          ..
                      }) => Ok(value.clone()),
            other => Err(syn::Error::new_spanned(other, "expected a string literal")),
        };

        let mut elements = tuple.elems.iter();

        match (elements.next(), elements.next(), elements.next()) {
            (Some(key), Some(value), None) => Ok((text(key)?, text(value)?)),
            _ => Err(syn::Error::new_spanned(
                tuple,
                "expected a `(\"key\", \"value\")` pair",
            )),
        }
    }

    pub fn add_bound(generics: &mut Generics, ty: &Type, bound: Path) {
        generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(#ty: #bound));
    }
}