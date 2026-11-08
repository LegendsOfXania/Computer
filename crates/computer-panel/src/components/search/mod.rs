use computer_model::{
    EntryDefinition,
    id::{EntryId, PageId},
    page::PageKind,
};
use dioxus::prelude::*;
use dioxus_i18n::tid;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::rc::Rc;

use crate::{
    components::utils::{icon::Icon, toast::ERR},
    search::{self, PAGE, SearchIndex},
    state::{app::AppState, ui::UiState},
    ws::Client,
};

const LIMIT: usize = 50;

#[derive(Clone, PartialEq)]
enum Target {
    Page(PageId),
    Entry(EntryId),
    NewEntry(String),
    NewPage,
}

impl Target {
    fn section(&self) -> usize {
        match self {
            Target::Entry(_) => 0,
            Target::NewEntry(_) => 1,
            Target::Page(_) | Target::NewPage => 2,
        }
    }
}

#[derive(Clone, PartialEq)]
struct Row {
    target: Target,
    icon: String,
    color: String,
    label: String,
    hint: String,
    meta: String,
}

fn fits(kind: &PageKind, definition: &EntryDefinition) -> bool {
    let tags = &definition.tags;
    tags.iter()
        .any(|tag| tag.eq_ignore_ascii_case(kind.as_str()))
}

fn new_page_row() -> Row {
    Row {
        target: Target::NewPage,
        icon: "plus".into(),
        color: String::new(),
        label: tid!("header-library-tooltip-create-page"),
        hint: String::new(),
        meta: String::new(),
    }
}

fn build(app: &AppState) -> SearchIndex<Row> {
    let (pages, entries, registry) = (app.pages.peek(), app.entries.peek(), app.registry.peek());
    let new_entry = tid!("search-new-entry");
    let row = |target, icon: &str, color: &str, label: &str, hint: &str, meta: &str| Row {
        target,
        icon: icon.into(),
        color: color.into(),
        label: label.into(),
        hint: hint.into(),
        meta: meta.into(),
    };

    let mut index = SearchIndex::new();

    let mut pages: Vec<_> = pages.values().collect();
    pages.sort_by(|a, b| (a.priority, &a.name).cmp(&(b.priority, &b.name)));

    for page in pages {
        let kind = page.kind.as_str();
        let target = Target::Page(page.id.clone());
        let chapter = page.chapter.as_deref();
        let place = chapter.unwrap_or("");
        let hit = row(target, &page.kind.icon(), "", &page.name, kind, &place);
        index.add_page(hit, &page.name, kind, page.id.as_str(), chapter);

        for id in &page.entries {
            let Some(entry) = entries.get(id) else {
                continue;
            };
            let (icon, color) = registry
                .get(&entry.kind)
                .map_or(("", ""), |d| (d.icon.as_str(), d.color.as_str()));

            let target = Target::Entry(id.clone());
            let hit = row(target, icon, color, entry.label(), &entry.kind, &page.name);
            index.add_entry(
                hit,
                entry.label(),
                &entry.kind,
                id.as_str(),
                Some(&page.name),
            );
        }
    }

    let mut definitions: Vec<_> = registry.values().collect();
    definitions.sort_by(|a, b| a.kind.cmp(&b.kind));

    for definition in definitions {
        let target = Target::NewEntry(definition.kind.clone());
        let (icon, color, kind) = (&definition.icon, &definition.color, &definition.kind);
        index.add_new_entry(
            row(target, icon, color, kind, &new_entry, ""),
            kind,
            &definition.tags,
        );
    }

    index
}

#[component]
pub fn Search(scope: u8, pick: Callback<EntryId>) -> Element {
    let (app, client, mut ui, toast) = (
        use_context::<AppState>(),
        use_context::<Client>(),
        use_context::<UiState>(),
        use_toast(),
    );

    let index = use_hook(|| Rc::new(build(&app)));
    let mut query = use_signal(String::new);
    let mut pending = use_signal(|| None::<EntryDefinition>);

    let sections = use_memo(move || {
        let (query, pending) = (query(), pending());
        let default = if pending.is_some() { PAGE } else { scope };
        let mut sections: [Vec<Row>; 3] = Default::default();

        for row in index.search(&query, default) {
            let accepted = match (&pending, &row.target) {
                (None, _) => true,
                (Some(definition), Target::Page(id)) => {
                    app.with_page(id, |page| fits(&page.kind, definition)) == Some(true)
                }
                _ => false,
            };
            let section = &mut sections[row.target.section()];

            if accepted && section.len() < LIMIT {
                section.push(row.clone());
            }
        }

        let page_fits = pending
            .as_ref()
            .is_none_or(|definition| PageKind::ALL.iter().any(|kind| fits(kind, definition)));

        if page_fits && search::scope(&query, default) & PAGE != 0 {
            sections[2].push(new_page_row());
        }

        sections
    });

    let fail = move |title: String, error: String| {
        toast.error(title, ToastOptions::new().description(error).duration(ERR));
    };

    let activate = use_callback(move |target: Target| {
        let client = client.clone();

        spawn(async move {
            let mut definition = pending();

            let page = match target {
                Target::Entry(id) => {
                    pick.call(id);
                    return ui.set_search(None);
                }

                Target::Page(id) if definition.is_none() => {
                    ui.set_opened_page(Some(id));
                    return ui.set_search(None);
                }
                Target::Page(id) => id,

                Target::NewEntry(kind) => {
                    let Some(wanted) = app.registry.read().get(&kind).cloned() else {
                        return;
                    };

                    let opened = ui.get_opened_page().filter(|id| {
                        app.with_page(id, |page| fits(&page.kind, &wanted)) == Some(true)
                    });

                    let Some(id) = opened else {
                        pending.set(Some(wanted));
                        return query.set(String::new());
                    };

                    definition = Some(wanted);
                    id
                }

                Target::NewPage => {
                    let kind = match &definition {
                        Some(wanted) => PageKind::ALL.into_iter().find(|kind| fits(kind, wanted)),
                        None => Some(PageKind::Sequence),
                    };
                    let Some(kind) = kind else {
                        return;
                    };

                    match client
                        .create_page(tid!("library-new-page-name"), kind)
                        .await
                    {
                        Ok(id) if definition.is_some() => id,
                        Ok(id) => {
                            ui.set_opened_page(Some(id));
                            return ui.set_search(None);
                        }
                        Err(error) => return fail(tid!("toast-create-page-failed"), error),
                    }
                }
            };

            let Some(definition) = definition else {
                return;
            };

            match client.create_entry(&definition, page).await {
                Ok(id) => {
                    pick.call(id);
                    ui.set_search(None);
                }
                Err(error) => fail(tid!("toast-create-entry-failed"), error),
            }
        });
    });

    let placeholder = match pending() {
        Some(definition) => tid!("search-pick-page", kind: definition.kind),
        None => tid!("header-search-placeholder"),
    };
    let titles = [
        tid!("search-entries"),
        tid!("search-new-entries"),
        tid!("search-pages"),
    ];
    let help = [
        ("!entry", titles[0].as_str()),
        ("!ne", titles[1].as_str()),
        ("!page", titles[2].as_str()),
        ("!id", "ID"),
        ("!kind", "Kind"),
    ];
    let sections = sections();
    let no_results = sections.iter().all(Vec::is_empty) && !query.read().is_empty();

    rsx! {
        document::Stylesheet { href: asset!("/assets/style/search/mod.css") }

        div { class: "search-overlay", onclick: move |_| ui.set_search(None),
            div { class: "search-box", onclick: move |event| event.stop_propagation(),
                div { class: "search-head",
                    label { class: "search-bar",
                        Icon { name: "search" }
                        input {
                            r#type: "text",
                            placeholder,
                            autocomplete: "off",
                            spellcheck: "false",
                            value: query,
                            onmounted: move |event| async move {
                                let _ = event.set_focus(true).await;
                            },
                            oninput: move |event| query.set(event.value()),
                        }
                    }

                    if pending().is_none() {
                        div { class: "search-help",
                            for (operator, label) in help {
                                span { key: "{operator}",
                                    b { "{operator}" }
                                    " {label}"
                                }
                            }
                        }
                    }
                }

                ul { class: "search-list",
                    if no_results {
                        li { class: "search-empty", {tid!("search-empty")} }
                    }

                    for (title, rows) in titles.iter().zip(&sections).filter(|(_, rows)| !rows.is_empty()) {
                        li { key: "{title}", class: "search-title", "{title}" }

                        for (index, row) in rows.iter().enumerate() {
                            li {
                                key: "{title}-{index}",
                                class: "search-row",
                                onclick: {
                                    let target = row.target.clone();
                                    move |_| activate.call(target.clone())
                                },
                                span { class: "search-row-icon", style: "color: {row.color}", Icon { name: row.icon.clone() } }
                                span { class: "search-row-label", "{row.label}" }
                                small { class: "search-row-hint", "{row.hint}" }
                                small { class: "search-row-meta", "{row.meta}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
