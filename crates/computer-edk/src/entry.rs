#[derive(Debug, Clone)]
pub struct EntryDefinition {
    pub name: &'static str,
    pub base: Option<&'static str>,
}