use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Fields, Variant, parse_macro_input, parse_quote,
};

use crate::util;

pub fn expand(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match expand_derive(input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

fn expand_derive(input: DeriveInput) -> syn::Result<TokenStream2> {
    let (kind, to_value, from_value) = match &input.data {
        Data::Struct(data) => expand_struct(data)?,
        Data::Enum(data) => expand_enum(data)?,
        Data::Union(data) => {
            return Err(syn::Error::new_spanned(
                data.union_token,
                "`Fields` cannot be derived for unions",
            ));
        }
    };

    let ident = input.ident;
    let mut generics = input.generics;

    for param in generics.type_params_mut() {
        param.bounds.push(parse_quote!(::computer_edk::Field));
    }

    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::computer_edk::Field for #ident #type_generics #where_clause {
            fn definition(name: &str) -> ::computer_edk::model::field::FieldDefinition {
                ::computer_edk::model::field::FieldDefinition {
                    name: name.into(),
                    kind: #kind,
                    flags: ::std::vec::Vec::new(),
                }
            }

            fn to_value(
                &self,
            ) -> Result<::computer_edk::model::value::Value, ::std::string::String> {
                #to_value
            }

            fn from_value(
                value: ::computer_edk::model::value::Value,
            ) -> Result<Self, ::std::string::String> {
                #from_value
            }
        }
    })
}

fn expand_struct(data: &DataStruct) -> syn::Result<(TokenStream2, TokenStream2, TokenStream2)> {
    match &data.fields {
        Fields::Named(fields) => {
            let definitions = util::named(fields);
            let mut to = Vec::new();
            let mut from = Vec::new();

            for field in &fields.named {
                let name = field
                    .ident
                    .as_ref()
                    .ok_or_else(|| syn::Error::new_spanned(field, "expected a named field"))?;
                let key = name.to_string();
                let ty = &field.ty;

                to.push(quote! {
                    map.insert(
                        ::std::string::String::from(#key),
                        ::computer_edk::Field::to_value(&self.#name)?,
                    );
                });

                from.push(quote! {
                    #name: <#ty as ::computer_edk::Field>::from_value(
                        map.shift_remove(#key)
                            .ok_or_else(|| format!("missing field `{}`", #key))?
                    )?
                });
            }

            Ok((
                quote! {
                    ::computer_edk::model::field::FieldKind::Struct(
                        ::std::vec![#(#definitions),*]
                    )
                },
                quote! {
                    let mut map = ::computer_edk::__private::IndexMap::new();
                    #(#to)*
                    Ok(::computer_edk::model::value::Value::Struct(map))
                },
                quote! {
                    let ::computer_edk::model::value::Value::Struct(mut map) = value else {
                        return Err("expected Struct".into());
                    };

                    let result = Self {
                        #(#from,)*
                    };

                    if let Some(name) = map.keys().next() {
                        return Err(format!("unexpected field `{name}`"));
                    }

                    Ok(result)
                },
            ))
        }
        Fields::Unnamed(fields) => {
            let definitions = util::unnamed(fields);
            let mut to = Vec::new();
            let mut from = Vec::new();

            for (index, field) in fields.unnamed.iter().enumerate() {
                let index = syn::Index::from(index);
                let key = index.index.to_string();
                let ty = &field.ty;

                to.push(quote! {
                    map.insert(
                        ::std::string::String::from(#key),
                        ::computer_edk::Field::to_value(&self.#index)?,
                    );
                });

                from.push(quote! {
                    <#ty as ::computer_edk::Field>::from_value(
                        map.shift_remove(#key)
                            .ok_or_else(|| format!("missing field `{}`", #key))?
                    )?
                });
            }

            Ok((
                quote! {
                    ::computer_edk::model::field::FieldKind::Struct(
                        ::std::vec![#(#definitions),*]
                    )
                },
                quote! {
                    let mut map = ::computer_edk::__private::IndexMap::new();
                    #(#to)*
                    Ok(::computer_edk::model::value::Value::Struct(map))
                },
                quote! {
                    let ::computer_edk::model::value::Value::Struct(mut map) = value else {
                        return Err("expected Struct".into());
                    };

                    let result = Self(#(#from,)*);

                    if let Some(name) = map.keys().next() {
                        return Err(format!("unexpected field `{name}`"));
                    }

                    Ok(result)
                },
            ))
        }
        Fields::Unit => Ok((
            quote! {
                ::computer_edk::model::field::FieldKind::Struct(
                    ::std::vec::Vec::new()
                )
            },
            quote! {
                Ok(::computer_edk::model::value::Value::Struct(
                    ::computer_edk::__private::IndexMap::new()
                ))
            },
            quote! {
                let ::computer_edk::model::value::Value::Struct(map) = value else {
                    return Err("expected Struct".into());
                };

                if let Some(name) = map.keys().next() {
                    return Err(format!("unexpected field `{name}`"));
                }

                Ok(Self)
            },
        )),
    }
}

fn expand_enum(data: &DataEnum) -> syn::Result<(TokenStream2, TokenStream2, TokenStream2)> {
    if data.variants.is_empty() {
        return Err(syn::Error::new_spanned(
            &data.variants,
            "`Fields` requires an enum with at least one variant",
        ));
    }

    let variants: Vec<_> = data.variants.iter().collect();
    let all_unit = variants
        .iter()
        .all(|variant| matches!(variant.fields, Fields::Unit));

    if all_unit {
        let names: Vec<_> = variants
            .iter()
            .map(|variant| variant.ident.to_string())
            .collect();

        let to = variants.iter().map(|variant| {
            let ident = &variant.ident;
            let name = ident.to_string();

            quote! {
                Self::#ident => Ok(
                    ::computer_edk::model::value::Value::Enum(
                        ::std::string::String::from(#name)
                    )
                )
            }
        });

        let from = variants.iter().map(|variant| {
            let ident = &variant.ident;
            let name = ident.to_string();

            quote! {
                #name => Ok(Self::#ident)
            }
        });

        return Ok((
            quote! {
                ::computer_edk::model::field::FieldKind::Enum(
                    ::std::vec![#(::std::string::String::from(#names)),*]
                )
            },
            quote! {
                match self {
                    #(#to),*
                }
            },
            quote! {
                let ::computer_edk::model::value::Value::Enum(name) = value else {
                    return Err("expected Enum".into());
                };

                match name.as_str() {
                    #(#from),*,
                    _ => Err(format!("unknown enum variant `{}`", name)),
                }
            },
        ));
    }

    let definitions: Vec<_> = variants
        .iter()
        .map(|variant| expand_variant_definition(variant))
        .collect();

    let mut to_arms = Vec::new();
    let mut from_arms = Vec::new();

    for variant in variants {
        let ident = &variant.ident;
        let variant_name = ident.to_string();

        match &variant.fields {
            Fields::Unit => {
                to_arms.push(quote! {
                    Self::#ident => Ok(
                        ::computer_edk::model::value::Value::Variant {
                            name: ::std::string::String::from(#variant_name),
                            fields: ::computer_edk::__private::IndexMap::new(),
                        }
                    )
                });

                from_arms.push(quote! {
                    #variant_name => {
                        if let Some(field) = fields.keys().next() {
                            return Err(format!(
                                "unexpected field `{field}` in variant `{}`",
                                #variant_name
                            ));
                        }

                        Ok(Self::#ident)
                    }
                });
            }
            Fields::Named(named) => {
                let mut names = Vec::new();
                let mut keys = Vec::new();
                let mut types = Vec::new();

                for field in &named.named {
                    let name = field
                        .ident
                        .clone()
                        .ok_or_else(|| syn::Error::new_spanned(field, "expected a named field"))?;

                    names.push(name.clone());
                    keys.push(name.to_string());
                    types.push(field.ty.clone());
                }

                let to = names.iter().zip(&keys).map(|(name, key)| {
                    quote! {
                        map.insert(
                            ::std::string::String::from(#key),
                            ::computer_edk::Field::to_value(#name)?,
                        );
                    }
                });

                let from = names
                    .iter()
                    .zip(&keys)
                    .zip(&types)
                    .map(|((name, key), ty)| {
                        quote! {
                            #name: <#ty as ::computer_edk::Field>::from_value(
                                fields.shift_remove(#key)
                                    .ok_or_else(|| format!(
                                        "missing field `{}` in variant `{}`",
                                        #key,
                                        #variant_name
                                    ))?
                            )?
                        }
                    });

                to_arms.push(quote! {
                    Self::#ident { #(#names),* } => {
                        let mut map = ::computer_edk::__private::IndexMap::new();
                        #(#to)*

                        Ok(::computer_edk::model::value::Value::Variant {
                            name: ::std::string::String::from(#variant_name),
                            fields: map,
                        })
                    }
                });

                from_arms.push(quote! {
                    #variant_name => {
                        let result = Self::#ident {
                            #(#from,)*
                        };

                        if let Some(field) = fields.keys().next() {
                            return Err(format!(
                                "unexpected field `{field}` in variant `{}`",
                                #variant_name
                            ));
                        }

                        Ok(result)
                    }
                });
            }
            Fields::Unnamed(unnamed) => {
                let mut names = Vec::new();
                let mut keys = Vec::new();
                let mut types = Vec::new();

                for (index, field) in unnamed.unnamed.iter().enumerate() {
                    names.push(format_ident!("field_{index}"));
                    keys.push(index.to_string());
                    types.push(field.ty.clone());
                }

                let to = names.iter().zip(&keys).map(|(name, key)| {
                    quote! {
                        map.insert(
                            ::std::string::String::from(#key),
                            ::computer_edk::Field::to_value(#name)?,
                        );
                    }
                });

                let from = keys.iter().zip(&types).map(|(key, ty)| {
                    quote! {
                        <#ty as ::computer_edk::Field>::from_value(
                            fields.shift_remove(#key)
                                .ok_or_else(|| format!(
                                    "missing field `{}` in variant `{}`",
                                    #key,
                                    #variant_name
                                ))?
                        )?
                    }
                });

                to_arms.push(quote! {
                    Self::#ident(#(#names),*) => {
                        let mut map = ::computer_edk::__private::IndexMap::new();
                        #(#to)*

                        Ok(::computer_edk::model::value::Value::Variant {
                            name: ::std::string::String::from(#variant_name),
                            fields: map,
                        })
                    }
                });

                from_arms.push(quote! {
                    #variant_name => {
                        let result = Self::#ident(#(#from,)*);

                        if let Some(field) = fields.keys().next() {
                            return Err(format!(
                                "unexpected field `{field}` in variant `{}`",
                                #variant_name
                            ));
                        }

                        Ok(result)
                    }
                });
            }
        }
    }

    Ok((
        quote! {
            ::computer_edk::model::field::FieldKind::Variant(
                ::std::vec![#(#definitions),*]
            )
        },
        quote! {
            match self {
                #(#to_arms),*
            }
        },
        quote! {
            let ::computer_edk::model::value::Value::Variant { name, fields } = value else {
                return Err("expected Variant".into());
            };

            let mut fields = fields;

            match name.as_str() {
                #(#from_arms),*,
                _ => Err(format!("unknown enum variant `{}`", name)),
            }
        },
    ))
}

fn expand_variant_definition(variant: &Variant) -> TokenStream2 {
    let name = variant.ident.to_string();

    let definitions = match &variant.fields {
        Fields::Unit => Vec::new(),
        Fields::Named(fields) => util::named(fields),
        Fields::Unnamed(fields) => util::unnamed(fields),
    };

    quote! {
        ::computer_edk::model::field::Variant {
            name: ::std::string::String::from(#name),
            fields: ::std::vec![#(#definitions),*],
        }
    }
}
