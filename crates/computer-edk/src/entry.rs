use computer_model::EntryDefinition;

pub trait Entry {
    fn definition() -> EntryDefinition;
}
