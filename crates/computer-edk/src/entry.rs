use computer_model::{EntryDefinition, value::Value};
use indexmap::IndexMap;

pub trait Entry: Sized {
    const KIND: &'static str;

    fn definition() -> EntryDefinition;

    fn to_fields(&self) -> Result<IndexMap<String, Value>, String>;

    fn from_fields(fields: IndexMap<String, Value>) -> Result<Self, String>;
}
