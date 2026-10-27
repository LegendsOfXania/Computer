use syn::{FieldsNamed, Ident, Meta, Type};

pub struct Base {
    pub name: Ident,
    pub ty: Type,
}

pub fn extract(fields: &mut FieldsNamed) -> syn::Result<Option<Base>> {
    let mut base = None;

    for field in &mut fields.named {
        let count = field
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("base"))
            .count();

        if count == 0 {
            continue;
        }

        if count > 1 {
            return Err(syn::Error::new_spanned(
                &*field,
                "the `#[base]` marker can only appear once per field",
            ));
        }

        let valid = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("base"))
            .is_some_and(|attr| matches!(&attr.meta, Meta::Path(_)));

        if !valid {
            return Err(syn::Error::new_spanned(
                &*field,
                "use `#[base]` without arguments",
            ));
        }

        if base.is_some() {
            return Err(syn::Error::new_spanned(
                &*field,
                "an entry can have only one `#[base]` field",
            ));
        }

        let name = field
            .ident
            .clone()
            .ok_or_else(|| syn::Error::new_spanned(&*field, "`#[base]` requires a named field"))?;

        let ty = field.ty.clone();
        field.attrs.retain(|attr| !attr.path().is_ident("base"));

        base = Some(Base { name, ty });
    }

    Ok(base)
}
