use computer_model::field::FieldDefinition;

#[derive(Debug, Clone)]
pub struct EntryBlueprint {
    pub kind: &'static str,
    pub base: Option<&'static str>,
    pub fields: Vec<FieldDefinition>,
    pub tags: &'static [&'static str],
}