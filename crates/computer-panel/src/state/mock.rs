use computer_model::{
    Entry, EntryDefinition, Library, Registry,
    field::{FieldDefinition, FieldKind, Variant},
    id::{EntryId, Identifier, PageId},
    page::{Page, PageKind},
    protocol::message::ServerMessage,
    value::Value,
};
use indexmap::IndexMap;

use super::app::AppState;

pub fn load(app: &mut AppState) {
    app.apply(ServerMessage::Registry(registry()));
    app.apply(ServerMessage::Library(library()));
}
fn field(name: &str, kind: FieldKind) -> FieldDefinition {
    FieldDefinition {
        name: name.into(),
        kind,
        flags: Vec::new(),
    }
}

fn triggers() -> FieldDefinition {
    field("triggers", FieldKind::List(Box::new(FieldKind::Reference)))
}

fn def(
    kind: &str,
    tags: &[&str],
    icon: &str,
    color: &str,
    fields: Vec<FieldDefinition>,
) -> EntryDefinition {
    let mut all = vec![field("name", FieldKind::String)];
    all.extend(fields);

    EntryDefinition {
        kind: kind.into(),
        tags: tags.iter().map(|tag| tag.to_string()).collect(),
        fields: all,
        icon: icon.into(),
        color: color.into(),
        version: 1,
    }
}

pub fn registry() -> Registry {
    let position = FieldKind::Struct(vec![
        field("x", FieldKind::Float),
        field("y", FieldKind::Float),
        field("z", FieldKind::Float),
        field("world", FieldKind::String),
    ]);

    let item = FieldKind::Variant(vec![
        Variant {
            name: "Vanilla".into(),
            fields: vec![field("id", FieldKind::String)],
        },
        Variant {
            name: "Custom".into(),
            fields: vec![
                field("source", FieldKind::String),
                field("model", FieldKind::Int),
            ],
        },
    ]);

    let color = FieldKind::Enum(
        ["info", "success", "warn", "error"]
            .into_iter()
            .map(String::from)
            .collect(),
    );

    let rewards = FieldKind::Map {
        key: Box::new(FieldKind::String),
        value: Box::new(FieldKind::Int),
    };

    Registry::new(vec![
        def(
            "trigger:player_join",
            &["trigger"],
            "algorithm",
            "#ff8205",
            vec![triggers()],
        ),
        def(
            "trigger:command",
            &["trigger"],
            "algorithm",
            "#ff8205",
            vec![field("command", FieldKind::String), triggers()],
        ),
        def(
            "basic:message",
            &["action"],
            "article",
            "#45ba82",
            vec![
                field("text", FieldKind::String),
                field("color", color),
                triggers(),
            ],
        ),
        def(
            "basic:teleport",
            &["action"],
            "git-commit",
            "#ffaf01",
            vec![
                field("position", position),
                field("instant", FieldKind::Bool),
                triggers(),
            ],
        ),
        def(
            "basic:give_item",
            &["action"],
            "plus",
            "#f05b62",
            vec![
                field("item", item),
                field("amount", FieldKind::Int),
                triggers(),
            ],
        ),
        def(
            "basic:fact",
            &["static", "fact"],
            "bulletlist",
            "#a1a1aa",
            vec![field("value", FieldKind::Int)],
        ),
        def(
            "basic:lootbox",
            &["static"],
            "folder",
            "#a1a1aa",
            vec![field("rewards", rewards)],
        ),
    ])
}

fn id(value: &str) -> EntryId {
    EntryId::from_string(value.into())
}

fn text(value: &str) -> Value {
    Value::String(value.into())
}

fn refs(keys: &[&str]) -> Value {
    Value::List(keys.iter().map(|key| Value::Reference(id(key))).collect())
}

fn map(pairs: Vec<(&str, Value)>) -> IndexMap<String, Value> {
    pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
}

fn entry(key: &str, kind: &str, fields: Vec<(&str, Value)>) -> Entry {
    Entry {
        id: id(key),
        kind: kind.into(),
        fields: map(fields),
        version: 1,
    }
}

fn page(
    key: &str,
    name: &str,
    kind: PageKind,
    priority: i32,
    chapter: Option<&str>,
    entries: &[&str],
) -> Page {
    Page {
        id: PageId::from_string(key.into()),
        kind,
        name: name.into(),
        priority,
        chapter: chapter.map(String::from),
        entries: entries.iter().map(|key| id(key)).collect(),
    }
}

pub fn library() -> Library {
    let mut entries = vec![
        entry(
            "join",
            "trigger:player_join",
            vec![
                ("name", text("Player join")),
                ("triggers", refs(&["welcome"])),
            ],
        ),
        entry(
            "welcome",
            "basic:message",
            vec![
                ("name", text("Welcome")),
                ("text", text("Welcome to the server!")),
                ("color", Value::Enum("success".into())),
                ("triggers", refs(&["spawn"])),
            ],
        ),
        entry(
            "spawn",
            "basic:teleport",
            vec![
                ("name", text("Go to spawn")),
                (
                    "position",
                    Value::Struct(map(vec![
                        ("x", Value::Float(0.5)),
                        ("y", Value::Float(64.0)),
                        ("z", Value::Float(-12.5)),
                        ("world", text("world")),
                    ])),
                ),
                ("instant", Value::Null),
                ("triggers", refs(&["sword"])),
            ],
        ),
        entry(
            "sword",
            "basic:give_item",
            vec![
                ("name", text("Starter sword CACAC CACACCAACCACACACACCACA")),
                (
                    "item",
                    Value::Variant {
                        name: "Vanilla".into(),
                        fields: map(vec![("id", text("minecraft:iron_sword"))]),
                    },
                ),
                ("amount", Value::Int(1)),
                ("triggers", refs(&[])),
            ],
        ),
        entry(
            "shop_cmd",
            "trigger:command",
            vec![
                ("name", text("/shop")),
                ("command", text("shop")),
                ("triggers", refs(&["shop_open"])),
            ],
        ),
        entry(
            "shop_open",
            "basic:message",
            vec![
                ("name", text("Shop open")),
                ("text", text("Take a look")),
                ("color", Value::Enum("info".into())),
                ("triggers", refs(&["sword"])),
            ],
        ),
        entry(
            "visits",
            "basic:fact",
            vec![("name", text("Visits")), ("value", Value::Int(0))],
        ),
        entry(
            "rank",
            "basic:fact",
            vec![("name", text("Rank")), ("value", Value::Int(3))],
        ),
        entry(
            "daily_box",
            "basic:lootbox",
            vec![
                ("name", text("Daily box")),
                (
                    "rewards",
                    Value::Map(vec![
                        (text("diamond"), Value::Int(1)),
                        (text("gold"), Value::Int(5)),
                    ]),
                ),
            ],
        ),
    ];

    let sandbox: Vec<String> = (0..12).map(|n| format!("sandbox{n:02}")).collect();

    for (n, key) in sandbox.iter().enumerate() {
        let next: Vec<&str> = sandbox.get(n + 1).map(String::as_str).into_iter().collect();

        entries.push(entry(
            key,
            "basic:message",
            vec![
                ("name", text(&format!("Step {n}"))),
                ("text", text("...")),
                ("color", Value::Enum("info".into())),
                ("triggers", refs(&next)),
            ],
        ));
    }

    let sandbox_keys: Vec<&str> = sandbox.iter().map(String::as_str).collect();

    let pages = vec![
        page(
            "intro",
            "Introduction",
            PageKind::Sequence,
            0,
            Some("story"),
            &["join", "welcome", "spawn", "sword"],
        ),
        page(
            "shop",
            "Shop",
            PageKind::Sequence,
            1,
            Some("story.shop"),
            &["shop_cmd", "shop_open"],
        ),
        page(
            "facts",
            "Facts",
            PageKind::Static,
            0,
            Some("config"),
            &["visits", "rank", "daily_box"],
        ),
        page(
            "sandbox",
            "Sandbox",
            PageKind::Sequence,
            2,
            None,
            &sandbox_keys,
        ),
        page("empty", "Empty", PageKind::Sequence, 5, None, &[]),
    ];

    Library {
        pages: pages.into_iter().map(|p| (p.id.clone(), p)).collect(),
        entries: entries.into_iter().map(|e| (e.id.clone(), e)).collect(),
    }
}
