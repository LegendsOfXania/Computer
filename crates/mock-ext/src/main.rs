use std::collections::{HashMap, HashSet};

use computer_edk::{entry, Field};
use computer_model::field::FieldKind;

#[derive(Field)]
enum QuestType {
    Main,
    Side,
    Daily,
}

#[derive(Field)]
enum Reward {
    Nothing,
    Experience {
        amount: i32,
    },
    Item {
        id: String,
        amount: u8,
    },
}

#[derive(Field)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

#[entry(
    kind = "quest",
    tags("test", "prout")
)]
struct Quest {
    name: String,
    level: u8,
    enabled: bool,
    quest_type: QuestType,
    reward: Reward,
    position: Position,
    tags: Vec<String>,
    unique_tags: HashSet<String>,
    metadata: HashMap<String, String>,
    description: Option<String>,
}

#[entry(
    kind = "special_quest",
    base = Quest
)]
struct SpecialQuest {
    special: bool,
}

fn main() {
    println!("=== Quest blueprint ===");

    let quest = Quest::blueprint();

    println!("{quest:#?}");

    println!("\n=== SpecialQuest blueprint ===");

    let special = SpecialQuest::blueprint();

    println!("{special:#?}");

    println!("\n=== Registry ===");

    let mut registry = computer_edk::registry::Registry::new();

    registry
        .register(quest)
        .expect("failed to register Quest");

    registry
        .register(special)
        .expect("failed to register SpecialQuest");

    let registry = registry
        .build()
        .expect("failed to build registry");

    println!("{registry:#?}");

    println!("\n=== Checks ===");

    let quest = registry
        .0
        .iter()
        .find(|entry| entry.kind == "quest")
        .expect("Quest missing");

    assert_eq!(quest.tags, ["test", "prout"]);
    assert_eq!(quest.fields.len(), 10);

    let quest_type = quest
        .fields
        .iter()
        .find(|field| field.name == "quest_type")
        .expect("quest_type missing");

    assert!(matches!(
        &quest_type.kind,
        FieldKind::Enum(variants)
            if variants == &vec![
                "Main".to_string(),
                "Side".to_string(),
                "Daily".to_string(),
            ]
    ));

    let reward = quest
        .fields
        .iter()
        .find(|field| field.name == "reward")
        .expect("reward missing");

    assert!(matches!(
        &reward.kind,
        FieldKind::Variant(_)
    ));

    let special = registry
        .0
        .iter()
        .find(|entry| entry.kind == "special_quest")
        .expect("SpecialQuest missing");

    assert_eq!(special.tags, Vec::<String>::new());
    assert_eq!(special.fields.len(), 11);

    assert!(special
        .fields
        .iter()
        .any(|field| field.name == "name"));

    assert!(special
        .fields
        .iter()
        .any(|field| field.name == "special"));

    println!("All EDK checks passed.");
}