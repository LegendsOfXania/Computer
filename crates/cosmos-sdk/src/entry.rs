use computer_model::{EntryDefinition, Field as ModelField, Node};

pub trait Taggable {
    fn tags() -> Vec<String>;
}

pub trait Entry: Sized + Taggable {
    fn fields(out: &mut Vec<ModelField>);

    fn read(node: Node<'_>) -> Result<Self, String>;
}

pub trait RegisteredEntry: Entry {
    const KIND: &'static str;
    const ICON: &'static str;
    const COLOR: &'static str;
    const VERSION: u32;

    #[must_use]
    fn definition() -> EntryDefinition {
        let mut fields = Vec::new();

        Self::fields(&mut fields);

        EntryDefinition {
            kind: Self::KIND.into(),
            tags: Self::tags(),
            fields,
            icon: Self::ICON.into(),
            color: Self::COLOR.into(),
            version: Self::VERSION,
        }
    }
}