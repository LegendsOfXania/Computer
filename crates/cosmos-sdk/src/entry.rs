use computer_model::{EntryDefinition, Field as ModelField, Node};

pub trait Entry: Sized {
    fn tags(out: &mut Vec<String>);

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
        let (mut tags, mut fields) = (Vec::new(), Vec::new());

        Self::tags(&mut tags);
        Self::fields(&mut fields);

        EntryDefinition {
            kind: Self::KIND.into(),
            tags,
            fields,
            icon: Self::ICON.into(),
            color: Self::COLOR.into(),
            version: Self::VERSION,
        }
    }
}
