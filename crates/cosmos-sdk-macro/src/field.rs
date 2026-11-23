use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, parse_quote};

use crate::common::{self, Member};

pub(super) fn expand(input: DeriveInput) -> syn::Result<TokenStream> {
    let members = common::members(&input, false, false)?;

    let field_count = u32::try_from(members.len())
        .map_err(|_| syn::Error::new_spanned(&input.ident, "too many struct fields"))?;

    let mut generics = input.generics.clone();

    for member in &members {
        common::add_bound(&mut generics, &member.ty, parse_quote!(::cosmos_sdk::Field));
    }

    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let definitions: Vec<_> = members.iter().map(define_member).collect();

    let locals: Vec<_> = (0..members.len())
        .map(|index| format_ident!("__field{index}"))
        .collect();

    let reads: Vec<_> = members
        .iter()
        .zip(&locals)
        .map(|(member, local)| read_member(member, local))
        .collect();

    let names = members.iter().map(|member| &member.ident);

    Ok(quote! {
        impl #impl_generics ::cosmos_sdk::Field
            for #ident #type_generics #where_clause
        {
            fn definition(
                name: &str,
                flags: &[(&str, &str)],
                out: &mut ::std::vec::Vec<::cosmos_sdk::FieldDefinition>,
            ) {
                ::cosmos_sdk::define_struct(name, flags, #field_count, out);

                #(#definitions)*
            }

            fn read(
                node: ::std::option::Option<::cosmos_sdk::Node<'_>>,
            ) -> ::std::result::Result<Self, ::std::string::String> {
                let node = ::cosmos_sdk::expect_struct(node)?;

                #(#reads)*

                ::std::result::Result::Ok(Self {
                    #(#names: #locals),*
                })
            }
        }
    })
}

fn define_member(member: &Member) -> TokenStream {
    let ty = &member.ty;
    let key = syn::LitStr::new(&member.key, member.ident.span());

    let flags = member.flags.iter().map(|(key, value)| {
        quote! { (#key, #value) }
    });

    quote! {
        ::cosmos_sdk::define::<#ty>(#key, &[#(#flags),*], out);
    }
}

fn read_member(member: &Member, local: &syn::Ident) -> TokenStream {
    let ty = &member.ty;
    let key = syn::LitStr::new(&member.key, member.ident.span());

    quote! {
        let #local = ::cosmos_sdk::member::<#ty>(node, #key)?;
    }
}
