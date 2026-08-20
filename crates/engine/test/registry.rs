use engine::{EntryDefinition, Registry, RegistryError, TagDefinition};
use model::{Field, PageType, Schema};

fn triggerable_tag() -> TagDefinition {
    TagDefinition::new("triggerable").field(Field::new("trigger", Schema::reference(["entry"])))
}

fn action_tag() -> TagDefinition {
    TagDefinition::new("action").field(Field::new("modifier", Schema::Text))
}

fn spoken_dialogue() -> EntryDefinition {
    EntryDefinition::new("spoken_dialogue")
        .tag("triggerable")
        .tag("action")
        .field(Field::new("text", Schema::Text))
        .field(Field::new("speaker", Schema::reference(["character"])))
}

#[test]
fn resolves_tag_fields_and_own_fields_in_order() {
    let registry = Registry::new();
    registry.register_tag(triggerable_tag()).unwrap();
    registry.register_tag(action_tag()).unwrap();
    registry.register_entry(spoken_dialogue()).unwrap();

    let resolved = registry.entries().get("spoken_dialogue").unwrap();

    assert_eq!(
        resolved
            .fields()
            .iter()
            .map(|field| field.name())
            .collect::<Vec<_>>(),
        ["name", "trigger", "modifier", "text", "speaker"],
    );
    assert_eq!(resolved.tags(), ["triggerable", "action"]);
}

#[test]
fn extensions_never_see_the_tag_registry() {
    // The whole point of `Registry::register_entry`: an extension only ever
    // deals with `EntryDefinition` and `Registry`, never a `TagRegistry`.
    let registry = Registry::new();
    registry.register_tag(triggerable_tag()).unwrap();

    let result = registry.register_entry(
        EntryDefinition::new("event").tag("triggerable"),
    );

    assert!(result.is_ok());
}

#[test]
fn rejects_unknown_tags() {
    let registry = Registry::new();

    let result = registry.register_entry(
        EntryDefinition::new("spoken_dialogue").tag("does_not_exist"),
    );

    assert_eq!(
        result,
        Err(RegistryError::UnknownTag {
            entry_type: "spoken_dialogue".to_owned(),
            tag: "does_not_exist".to_owned(),
        }),
    );
}

#[test]
fn rejects_duplicate_tag_registration() {
    let registry = Registry::new();
    registry.register_tag(triggerable_tag()).unwrap();

    let result = registry.register_tag(triggerable_tag());

    assert_eq!(
        result,
        Err(RegistryError::DuplicateTag("triggerable".to_owned())),
    );
}

#[test]
fn rejects_duplicate_entry_registration() {
    let registry = Registry::new();
    registry.register_entry(EntryDefinition::new("dialogue")).unwrap();

    let result = registry.register_entry(EntryDefinition::new("dialogue"));

    assert_eq!(
        result,
        Err(RegistryError::DuplicateEntry("dialogue".to_owned())),
    );
}

#[test]
fn rejects_field_collisions_between_two_tags() {
    let registry = Registry::new();
    registry
        .register_tag(TagDefinition::new("a").field(Field::new("shared", Schema::Text)))
        .unwrap();
    registry
        .register_tag(TagDefinition::new("b").field(Field::new("shared", Schema::Text)))
        .unwrap();

    let result = registry.register_entry(
        EntryDefinition::new("conflicted")
            .tag("a")
            .tag("b"),
    );

    assert_eq!(
        result,
        Err(RegistryError::FieldCollision {
            entry_type: "conflicted".to_owned(),
            field: "shared".to_owned(),
        }),
    );
}

#[test]
fn rejects_own_field_colliding_with_the_universal_name_field() {
    let registry = Registry::new();

    let result = registry.register_entry(
        EntryDefinition::new("dialogue")
            .field(Field::new("name", Schema::Text)),
    );

    assert_eq!(
        result,
        Err(RegistryError::FieldCollision {
            entry_type: "dialogue".to_owned(),
            field: "name".to_owned(),
        }),
    );
}

#[test]
fn a_partially_resolved_entry_is_not_left_registered_after_a_collision() {
    let registry = Registry::new();
    let result = registry.register_entry(
        EntryDefinition::new("dialogue")
            .field(Field::new("name", Schema::Text)),
    );

    assert!(result.is_err());
    assert!(!registry.entries().contains("dialogue"));
}