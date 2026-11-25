use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::common;

pub(super) fn expand(input: DeriveInput) -> syn::Result<TokenStream> {
    let members = common::members(&input, false, false)?;

    let field_count = u32::try_from(members.len())
        .map_err(|_| syn::Error::new_spanned(&input.ident, "too many struct fields"))?;

    let generics = common::bounded(&input.generics, &members);
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let ident = &input.ident;
    let defines = members.iter().map(common::define);
    let reads = members.iter().map(common::read);
    let names = members.iter().map(|member| &member.ident);

    Ok(quote! {
        impl #impl_generics ::cosmos_sdk::Field for #ident #type_generics #where_clause {
            fn definition(
                name: &str,
                flags: &[(&str, &str)],
                out: &mut ::std::vec::Vec<::cosmos_sdk::computer_model::Field>,
            ) {
                ::cosmos_sdk::__private::define_struct(name, flags, #field_count, out);

                #(#defines)*
            }

            fn read(
                node: ::std::option::Option<::cosmos_sdk::Node<'_>>,
            ) -> ::std::result::Result<Self, ::std::string::String> {
                let node = ::cosmos_sdk::__private::expect_struct(node)?;

                ::std::result::Result::Ok(Self { #(#names: #reads),* })
            }
        }
    })
}