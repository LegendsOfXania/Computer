mod category;
mod hit;

use computer_model::{
    EntryDefinition,
    id::{EntryId, Identifier, PageId},
    page::{Page, PageKind},
};
use dioxus::prelude::*;
use dioxus_i18n::tid;
use dioxus_primitives::toast::{ToastOptions, use_toast};

use category::Category;

use crate::{
    components::utils::{icon::Icon, toast::ERR},
    search::{self, ENTRY, Fields, NEW_ENTRY, PAGE, SearchIndex},
    state::{app::AppState, ui::UiState},
    ws::Client,
};
use crate::search::PAGE_PICK;

const LIMIT: usize = 50;

type Sections = [Vec<SearchItem>; 3];

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
            Self::Entry(_) => 0,
            Self::NewEntry(_) => 1,
            Self::Page(_) | Self::NewPage => 2,
        }
    }
}

#[derive(Clone, PartialEq)]
struct SearchItem {
    target: Target,
    icon: String,
    color: String,
    label: String,
    hint: String,
    meta: String,
}

impl SearchItem {
    fn new(target: Target, icon: &str, color: &str, label: &str, hint: &str, meta: &str) -> Self {
        Self {
            target,
            icon: icon.into(),
            color: color.into(),
            label: label.into(),
            hint: hint.into(),
            meta: meta.into(),
        }
    }
}

fn fits(kind: &PageKind, definition: &EntryDefinition) -> bool {
    definition.tags.iter().any(|tag| tag.eq_ignore_ascii_case(kind.as_str()))
}

fn build_index(app: &AppState) -> SearchIndex<SearchItem> {
    let (pages, entries, registry) = (
        app.pages.read(),
        app.entries.read(),
        app.registry.read(),
    );

    let new_entry = tid!("search-new-entry");
    let mut index = SearchIndex::new();

    let mut ordered: Vec<&Page> = pages.values().collect();
    ordered.sort_by(|a, b| (a.priority, &a.name).cmp(&(b.priority, &b.name)));

    for page in ordered {
        let chapter = page.chapter.as_deref().unwrap_or("");
        let kind = page.kind.as_str();

        let target = Target::Page(page.id.clone());
        let item = SearchItem::new(
            target,
            page.kind.icon(),
            "",
            &page.name,
            kind,
            chapter,
        );

        let fields = Fields {
            name: &page.name,
            kind,
            id: page.id.as_str(),
            chapter,
            ..Fields::default()
        };

        index.add(PAGE, item, fields);

        let place = if chapter.is_empty() {
            page.name.clone()
        } else {
            format!("{chapter} · {}", page.name)
        };

        for id in &page.entries {
            let Some(entry) = entries.get(id) else {
                continue;
            };

            let definition = registry.get(&entry.kind);
            let (icon, color) =
                definition.map_or(("", ""), |d| (d.icon.as_str(), d.color.as_str()));

            let target = Target::Entry(id.clone());
            let item = SearchItem::new(
                target,
                icon,
                color,
                entry.label(),
                &entry.kind,
                &place,
            );

            let fields = Fields {
                name: entry.label(),
                kind: &entry.kind,
                id: id.as_str(),
                chapter,
                parent: &page.name,
                tags: definition.map_or(&[], |d| d.tags.as_slice()),
            };

            index.add(ENTRY, item, fields);
        }
    }

    let mut definitions: Vec<_> = registry.values().collect();
    definitions.sort_by(|a, b| a.kind.cmp(&b.kind));

    for definition in definitions {
        let kind = &definition.kind;
        let target = Target::NewEntry(kind.clone());
        let extension = search::extension(kind);

        let item = SearchItem::new(
            target,
            &definition.icon,
            &definition.color,
            kind,
            &new_entry,
            extension,
        );

        let fields = Fields {
            name: kind,
            kind,
            tags: &definition.tags,
            ..Fields::default()
        };

        index.add(NEW_ENTRY, item, fields);
    }

    index
}

fn new_page_item() -> SearchItem {
    let label = tid!("header-library-tooltip-create-page");

    SearchItem::new(
        Target::NewPage,
        "plus",
        "",
        &label,
        "",
        "",
    )
}

fn build_sections(
    app: &AppState,
    index: &SearchIndex<SearchItem>,
    query: &str,
    scope: u8,
    definition: Option<&EntryDefinition>,
) -> Sections {
    let default = if definition.is_some() { PAGE } else { scope };
    let pages = app.pages.read();
    let mut sections = Sections::default();

    for item in index.search(query, default) {
        if let Some(definition) = definition {
            let Target::Page(id) = &item.target else {
                continue;
            };

            if !pages.get(id).is_some_and(|page| fits(&page.kind, definition)) {
                continue;
            }
        }

        let items = &mut sections[item.target.section()];
        if items.len() < LIMIT {
            items.push(item.clone());

            if sections.iter().all(|items| items.len() == LIMIT) {
                break;
            }
        }
    }

    let can_create_page = definition.is_none_or(|definition| {
        PageKind::ALL.iter().any(|kind| fits(kind, definition))
    });

    if can_create_page && search::scope(query, default) & PAGE != 0 {
        sections[2].push(new_page_item());
    }

    sections
}

#[component]
pub fn Search(scope: u8, pick: Callback<EntryId>) -> Element {
    let (app, client, mut ui, toast) = (
        use_context::<AppState>(),
        use_context::<Client>(),
        use_context::<UiState>(),
        use_toast(),
    );

    let fail = use_callback(move |(title, error): (String, String)| {
        toast.error(title, ToastOptions::new().description(error).duration(ERR));
    });

    let page_pick = scope & PAGE_PICK != 0;
    let scope = scope & !PAGE_PICK;

    let page_definition = if page_pick {
        ui.get_opened_entry().and_then(|id| {
            app.with_entry(&id, |entry| {
                app.registry.read().get(&entry.kind).cloned()
            })
                .flatten()
        })
    } else {
        None
    };

    let index = use_memo(move || build_index(&app));
    let mut query = use_signal(String::new);
    let mut pending = use_signal(|| None::<EntryDefinition>);
    let mut collapsed = use_signal(|| [false; 3]);

    let filter = page_definition.clone();

    let sections = use_memo(move || {
        let definition = pending.read().clone().or_else(|| filter.clone());

        build_sections(
            &app,
            &index.read(),
            &query.read(),
            scope,
            definition.as_ref(),
        )
    });

    let activate = use_callback(move |target: Target| {
        let client = client.clone();
        let page_definition = page_definition.clone();

        spawn(async move {
            let mut definition = pending();

            let page = match target {
                Target::Entry(id) => {
                    pick.call(id);
                    return ui.set_search(None);
                }

                Target::Page(id) if definition.is_none() => {
                    if page_pick {
                        pick.call(EntryId::from_string(id.into_string()));
                    } else {
                        ui.set_opened_page(Some(id));
                    }

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
                        Some(wanted) => {
                            PageKind::ALL.into_iter().find(|kind| fits(kind, wanted))
                        }

                        None if page_pick => {
                            page_definition.as_ref().and_then(|wanted| {
                                PageKind::ALL.into_iter().find(|kind| fits(kind, wanted))
                            })
                        }

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

                        Ok(id) if page_pick => {
                            pick.call(EntryId::from_string(id.into_string()));
                            return ui.set_search(None);
                        }

                        Ok(id) => {
                            ui.set_opened_page(Some(id));
                            return ui.set_search(None);
                        }

                        Err(error) => {
                            return fail.call((tid!("toast-create-page-failed"), error));
                        }
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

                Err(error) => fail.call((tid!("toast-create-entry-failed"), error)),
            }
        });
    });

    let fold = use_callback(move |category: usize| {
        collapsed.with_mut(|values| values[category] = !values[category]);
    });

    let select = use_callback(move |(category, item): (usize, usize)| {
        let target = sections.read()[category]
            .get(item)
            .map(|item| item.target.clone());

        if let Some(target) = target {
            activate.call(target);
        }
    });

    let titles = [
        tid!("search-existing-entries"),
        tid!("search-new-entries"),
        tid!("search-pages"),
    ];

    let (kind_title, extension_title, chapter_title) = (
        tid!("search-kind"),
        tid!("search-extension"),
        tid!("search-chapter"),
    );

    let placeholder = match pending.read().as_ref() {
        Some(definition) => {
            tid!("search-pick-page", kind: definition.kind.clone())
        }
        None => tid!("header-search-placeholder"),
    };

    let text = query();
    let parts = search::highlight(&text);
    let sections = sections();
    let collapsed = collapsed();

    let no_results = sections.iter().all(Vec::is_empty);

    rsx! {
        Stylesheet { href: asset!("/assets/style/search/mod.css") }

        div {
            class: "search-overlay",
            onclick: move |_| ui.set_search(None),

            div {
                class: "search-box",
                onclick: move |event| event.stop_propagation(),

                div { class: "search-head",
                    label { class: "search-bar",
                        Icon { name: "search" }

                        div { class: "search-input-scroll",
                            div { class: "search-input-track",
                                span { class: "search-query-render",
                                    if text.is_empty() {
                                        span {
                                            class: "search-placeholder",
                                            "{placeholder}"
                                        }
                                    }

                                    for (index, (part, operator)) in parts.into_iter().enumerate() {
                                        span {
                                            key: "{index}",
                                            class: if operator {
                                                "search-query-operator"
                                            } else {
                                                ""
                                            },
                                            "{part}"
                                        }
                                    }
                                }

                                input {
                                    r#type: "text",
                                    class: "search-input",
                                    aria_label: "{placeholder}",
                                    autocomplete: "off",
                                    spellcheck: "false",
                                    value: "{text}",

                                    onmounted: move |event| async move {
                                        let _ = event.set_focus(true).await;
                                    },

                                    oninput: move |event| query.set(event.value()),
                                }
                            }
                        }
                    }

                    if pending.read().is_none() && !page_pick {
                        div { class: "search-help",
                            span { b { "!entry" } " {titles[0]}" }
                            span { b { "!ne" } " {titles[1]}" }
                            span { b { "!page" } " {titles[2]}" }
                            span { b { "!id" } " ID" }
                            span { b { "!kind:" } " {kind_title}" }
                            span { b { "!ext:" } " {extension_title}" }
                            span { b { "!chapter:" } " {chapter_title}" }
                        }
                    }
                }

                div { class: "search-list",
                    if no_results {
                        div {
                            class: "search-empty",
                            {tid!("search-empty")}
                        }
                    }

                    for (category, items) in sections.into_iter().enumerate() {
                        if !items.is_empty() {
                            Category {
                                key: "{category}",
                                index: category,
                                title: titles[category].clone(),
                                items,
                                collapsed: collapsed[category],
                                ontoggle: fold,
                                onselect: select,
                            }
                        }
                    }
                }
            }
        }
    }
}