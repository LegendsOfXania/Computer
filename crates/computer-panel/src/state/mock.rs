// TEMPORAIRE : données factices pour développer le graphe. À supprimer.

use computer_model::{
    entry::Entry,
    key::EntryKey,
    page::{Page, PageKind},
    value::Value,
};
use dioxus::signals::WritableExt;
use indexmap::IndexMap;

use crate::state::{app::AppState, ui::UiState};

fn entry(page: u64, id: u64, kind: &str, name: &str, next: &[(u64, u64)]) -> Entry {
    let mut fields = IndexMap::new();
    fields.insert("name".to_string(), Value::String(name.to_string()));
    fields.insert(
        "next".to_string(),
        Value::List(
            next.iter()
                .map(|&(p, e)| Value::Reference(EntryKey::new(p, e)))
                .collect(),
        ),
    );

    Entry {
        key: EntryKey::new(page, id),
        kind: kind.to_string(),
        version: 0,
        fields,
    }
}

fn page(id: u64, name: &str, kind: PageKind, entries: &[Entry]) -> Page {
    Page {
        id,
        name: name.to_string(),
        kind,
        priority: id as i32,
        chapter: Some("demo".to_string()),
        entries: entries.iter().map(|e| e.key).collect(),
    }
}

pub fn seed(app: AppState, ui: UiState) {
    let p1 = [
        entry(
            1,
            1,
            "start",
            "Start eeeeeeeeeeeeeeeeeeee",
            &[(1, 2), (1, 3)],
        ),
        entry(1, 2, "say", "Welcome", &[(1, 4)]),
        entry(1, 3, "say", "Branch B", &[(1, 4)]),
        entry(1, 4, "give_item", "Reward", &[(3, 1)]), // vers une autre page
        entry(1, 5, "lookup", "Lookup", &[(2, 1)]),    // vers une page Static : pas d'edge
    ];
    let p2 = [
        entry(2, 1, "fact", "Quest started", &[]),
        entry(2, 2, "fact", "Gold", &[(2, 1)]), // page Static : pas d'edge
    ];
    let p3 = [
        entry(3, 1, "say", "Next page", &[(3, 2)]),
        entry(3, 2, "teleport", "Go", &[]),
    ];

    let pages = [
        page(1, "Intro", PageKind::Sequence, &p1),
        page(2, "Constants", PageKind::Static, &p2),
        page(3, "Chapter 2", PageKind::Sequence, &p3),
    ];

    let mut app_pages = app.pages;
    let mut app_entries = app.entries;
    let mut opened_page = ui.page;

    app_pages.set(pages.into_iter().map(|p| (p.id, p)).collect());
    app_entries.set(
        p1.into_iter()
            .chain(p2)
            .chain(p3)
            .map(|e| (e.key, e))
            .collect(),
    );
    opened_page.set(Some(1));
}
