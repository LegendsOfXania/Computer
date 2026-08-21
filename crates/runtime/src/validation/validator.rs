use super::{ValidationDiagnostic, ValidationError};
use crate::{EntryKey, Library, Registry};
use model::{EntryData, ReferenceSchema, Schema, Value};
use std::str::FromStr;

pub struct Validator<'a> {
    registry: &'a Registry,
    library: &'a Library,
}
impl<'a> Validator<'a> {
    pub fn new(registry: &'a Registry, library: &'a Library) -> Self {
        Self { registry, library }
    }
    pub fn validate_library(&self) -> Vec<ValidationDiagnostic> {
        self.library
            .pages()
            .into_iter()
            .flat_map(|page| self.validate_page(&page))
            .collect()
    }
    pub fn validate_page(&self, page: &crate::Page) -> Vec<ValidationDiagnostic> {
        let mut diagnostics = Vec::new();

        for (entry_id, entry) in page.iter() {
            diagnostics.extend(
                self.validate_entry(entry.data())
                    .into_iter()
                    .map(|error| ValidationDiagnostic::new(page.id(), entry_id, error)),
            );
        }

        diagnostics
    }

    pub fn validate_entry(&self, entry: &EntryData) -> Vec<ValidationError> {
        let Some(definition) = self.registry.entries().get(entry.entry_type()) else {
            return vec![ValidationError::UnknownEntryType {
                entry_type: entry.entry_type().to_owned(),
            }];
        };
        let mut errors = Vec::new();
        for name in entry.fields().keys() {
            if definition.fields().iter().all(|field| field.name() != name) {
                errors.push(ValidationError::UnknownField {
                    field: name.clone(),
                });
            }
        }
        for field in definition.fields() {
            let value = match entry.get(field.name()) {
                Some(v) => v,
                None => {
                    if !field.schema().accepts(&Value::Null) {
                        errors.push(ValidationError::MissingField {
                            field: field.name().to_owned(),
                        });
                    }
                    continue;
                }
            };

            if !field.schema().accepts(value) {
                errors.push(ValidationError::InvalidValue {
                    field: field.name().to_owned(),
                });
                continue;
            }

            self.validate_value(field.name(), field.schema(), value, &mut errors);
        }
        errors
    }
    fn validate_value(
        &self,
        path: &str,
        schema: &Schema,
        value: &Value,
        errors: &mut Vec<ValidationError>,
    ) {
        match (schema, value) {
            (Schema::Reference(schema), Value::Reference(reference)) => {
                self.validate_reference(path, schema, reference, errors)
            }
            (Schema::Struct(schema), Value::Struct(values)) => {
                for field in schema.fields() {
                    if let Some(value) = values.get(field.name()) {
                        let path = format!("{path}.{}", field.name());
                        self.validate_value(&path, field.schema(), value, errors);
                    }
                }
            }
            (Schema::List(schema), Value::List(values)) => {
                for (index, value) in values.iter().enumerate() {
                    let path = format!("{path}[{index}]");
                    self.validate_value(&path, schema, value, errors);
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
        let Ok(key) = EntryKey::from_str(reference) else {
            errors.push(ValidationError::InvalidReference {
                field: field.to_owned(),
                reference: reference.to_owned(),
            });
            return;
        };
        let Some(entry) = self.library.entries().get(&key) else {
            errors.push(ValidationError::UnknownReference {
                field: field.to_owned(),
                reference: reference.to_owned(),
            });
            return;
        };
        if let Some(expected) = schema.entry_type() {
            if entry.entry_type() != expected {
                errors.push(ValidationError::InvalidReferenceEntryType {
                    field: field.to_owned(),
                    expected: expected.to_owned(),
                    actual: entry.entry_type().to_owned(),
                });
            }
        }
        let Some(definition) = self.registry.entries().get(entry.entry_type()) else {
            return;
        };
        for tag in schema.tags() {
            if !definition.tags().iter().any(|candidate| candidate == tag) {
                errors.push(ValidationError::MissingReferenceTag {
                    field: field.to_owned(),
                    tag: tag.clone(),
                });
            }
        }
    }
}
