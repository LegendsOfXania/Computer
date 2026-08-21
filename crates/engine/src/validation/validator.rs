use model::{
    EntryData, Schema, Value, schema::ReferenceSchema,
};

use runtime::{
    EntryKey,
    Library,
    Page,
};

use crate::{
    Registry, ValidationError, validation::ValidationDiagnostic,
};

pub struct Validator<'a> {
    registry: &'a Registry,
    library: &'a Library,
}

impl<'a> Validator<'a> {
    #[inline]
    pub fn new(
        registry: &'a Registry,
        library: &'a Library,
    ) -> Self {
        Self {
            registry,
            library,
        }
    }

    pub fn validate_library(
        &self,
    ) -> Vec<ValidationDiagnostic> {
        self.library
            .pages()
            .into_iter()
            .flat_map(|page| self.validate_page(&page))
            .collect()
    }

    pub fn validate_page(
        &self,
        page: &Page,
    ) -> Vec<ValidationDiagnostic> {
        let page_id = page.id();

        page
            .entries()
            .flat_map(|entry| {
                self.validate_entry(entry.data())
                    .into_iter()
                    .map(|error| {
                        ValidationDiagnostic::new(
                            EntryKey::new(page_id, entry.id()),
                            error,
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    pub fn validate_entry(
        &self,
        entry: &EntryData,
    ) -> Vec<ValidationError> {
        let Some(definition) = self
            .registry
            .entries()
            .get(entry.entry_type())
        else {
            return vec![
                ValidationError::UnknownEntryType {
                    entry_type: entry.entry_type().to_owned(),
                },
            ];
        };

        let mut errors = Vec::new();

        for name in entry.fields().keys() {
            if definition
                .fields()
                .iter()
                .all(|field| field.name() != name)
            {
                errors.push(
                    ValidationError::UnknownField {
                        field: name.clone(),
                    },
                );
            }
        }

        for field in definition.fields() {
            let Some(value) = entry.field(field.name()) else {
                errors.push(
                    ValidationError::MissingField {
                        field: field.name().to_owned(),
                    },
                );

                continue;
            };

            if !field.schema().accepts(value) {
                errors.push(
                    ValidationError::InvalidValue {
                        field: field.name().to_owned(),
                    },
                );

                continue;
            }

            self.validate_value(
                field.name(),
                field.schema(),
                value,
                &mut errors,
            );
        }

        errors
    }

    fn validate_value(
        &self,
        field: &str,
        schema: &Schema,
        value: &Value,
        errors: &mut Vec<ValidationError>,
    ) {
        match (schema, value) {
            (
                Schema::Reference(schema),
                Value::Reference(reference),
            ) => {
                self.validate_reference(
                    field,
                    schema,
                    reference,
                    errors,
                );
            }

            (
                Schema::Struct(schema),
                Value::Struct(values),
            ) => {
                for nested_field in schema.fields() {
                    let Some(value) =
                        values.get(nested_field.name())
                    else {
                        continue;
                    };

                    let field = format!(
                        "{field}.{}",
                        nested_field.name(),
                    );

                    self.validate_value(
                        &field,
                        nested_field.schema(),
                        value,
                        errors,
                    );
                }
            }

            (
                Schema::List(schema),
                Value::List(values),
            ) => {
                for (index, value) in
                    values.iter().enumerate()
                {
                    let field =
                        format!("{field}[{index}]");

                    self.validate_value(
                        &field,
                        schema,
                        value,
                        errors,
                    );
                }
            }

            _ => {}
        }
    }

    fn validate_reference(
        &self,
        field: &str,
        schema: &ReferenceSchema,
        reference: &str,
        errors: &mut Vec<ValidationError>,
    ) {
        let Some(key) = EntryKey::parse(reference) else {
            errors.push(
                ValidationError::InvalidReference {
                    field: field.to_owned(),
                    reference: reference.to_owned(),
                },
            );

            return;
        };

        let Some(entry) =
            self.library.entries().get(&key)
        else {
            errors.push(
                ValidationError::UnknownReference {
                    field: field.to_owned(),
                    reference: reference.to_owned(),
                },
            );

            return;
        };

        if let Some(expected) = schema.entry_type() {
            if entry.entry_type() != expected {
                errors.push(
                    ValidationError::InvalidReferenceEntryType {
                        field: field.to_owned(),
                        expected: expected.to_owned(),
                        actual: entry
                            .entry_type()
                            .to_owned(),
                    },
                );
            }
        }

        let Some(definition) = self
            .registry
            .entries()
            .get(entry.entry_type())
        else {
            return;
        };

        for tag in schema.tags() {
            if definition
                .tags()
                .iter()
                .all(|candidate| candidate != tag)
            {
                errors.push(
                    ValidationError::MissingReferenceTag {
                        field: field.to_owned(),
                        tag: tag.clone(),
                    },
                );
            }
        }
    }
}