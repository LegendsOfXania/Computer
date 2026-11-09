mod category;
mod result;

use computer_model::{
    EntryDefinition,
    id::{EntryId, PageId},
    page::{Page, PageKind},
};
use dioxus::prelude::*;
use dioxus_i18n::tid;
use dioxus_primitives::toast::{ToastOptions, use_toast};

use crate::{
    components::{
        search::category::Category,
        utils::{icon::Icon, toast::ERR},
    },
    search::{self, PAGE, SearchIndex, extension_for_kind},
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum Selection {
    Category(usize),
    Result(usize, usize),
}

fn fits(kind: &PageKind, definition: &EntryDefinition) -> bool {
    definition
        .tags
        .iter()
        .any(|tag| tag.eq_ignore_ascii_case(kind.as_str()))
}

fn new_page_item() -> SearchItem {
    SearchItem {
        target: Target::NewPage,
        icon: "plus".into(),
        color: String::new(),
        label: tid!("header-library-tooltip-create-page"),
        hint: String::new(),
        meta: String::new(),
    }
}

fn build_index(app: &AppState) -> SearchIndex<Target> {
    let pages = app.pages.read();
    let entries = app.entries.read();
    let registry = app.registry.read();

    let mut ordered_pages: Vec<&Page> = pages.values().collect();
    ordered_pages.sort_by(|a, b| (a.priority, &a.name).cmp(&(b.priority, &b.name)));

    let mut index = SearchIndex::new();

    for page in ordered_pages {
        index.add_page(
            Target::Page(page.id.clone()),
            &page.name,
            page.kind.as_str(),
            page.id.as_str(),
            page.chapter.as_deref(),
        );

        for id in &page.entries {
            let Some(entry) = entries.get(id) else {
                continue;
            };
            let definition = registry.get(&entry.kind);
            let extension = extension_for_kind(&entry.kind);
            let tags = definition.map_or(&[][..], |definition| definition.tags.as_slice());

            index.add_entry(
                Target::Entry(id.clone()),
                entry.label(),
                &entry.kind,
                id.as_str(),
                extension,
                page.chapter.as_deref(),
                Some(&page.name),
                tags,
            );
        }
    }

    let mut definitions: Vec<_> = registry.values().collect();
    definitions.sort_by(|a, b| a.kind.cmp(&b.kind));

    for definition in definitions {
        index.add_new_entry(
            Target::NewEntry(definition.kind.clone()),
            &definition.kind,
            extension_for_kind(&definition.kind),
            &definition.tags,
        );
    }

    index
}

fn display_item(app: &AppState, target: Target) -> Option<SearchItem> {
    match &target {
        Target::Page(id) => app.with_page(id, |page| SearchItem {
            target: target.clone(),
            icon: page.kind.icon().into(),
            color: String::new(),
            label: page.name.clone(),
            hint: page.kind.as_str().into(),
            meta: page.chapter.clone().unwrap_or_default(),
        }),
        Target::Entry(id) => app.with_entry(id, |entry| {
            let registry = app.registry.read();
            let (icon, color) = registry
                .get(&entry.kind)
                .map_or((String::new(), String::new()), |definition| {
                    (definition.icon.clone(), definition.color.clone())
                });

            let pages = app.pages.read();
            let parent = pages.values().find(|page| page.entries.contains(id));
            let meta = parent.map_or_else(String::new, |page| match page.chapter.as_deref() {
                Some(chapter) if !chapter.is_empty() => {
                    format!("{} · {}", chapter, page.name)
                }
                _ => page.name.clone(),
            });

            SearchItem {
                target: target.clone(),
                icon,
                color,
                label: entry.label().to_owned(),
                hint: entry.kind.clone(),
                meta,
            }
        }),
        Target::NewEntry(kind) => app.registry.read().get(kind).map(|definition| SearchItem {
            target: target.clone(),
            icon: definition.icon.clone(),
            color: definition.color.clone(),
            label: definition.kind.clone(),
            hint: tid!("search-new-entry"),
            meta: extension_for_kind(&definition.kind).to_owned(),
        }),
        Target::NewPage => Some(new_page_item()),
    }
}

fn build_sections(
    app: &AppState,
    index: &SearchIndex<Target>,
    query: &str,
    scope: u8,
    pending: Option<&EntryDefinition>,
) -> [Vec<SearchItem>; 3] {
    let default = if pending.is_some() { PAGE } else { scope };
    let mut sections: [Vec<SearchItem>; 3] = std::array::from_fn(|_| Vec::new());

    for matched in index.search(query, default) {
        let target = matched.target.clone();

        if let Some(definition) = pending {
            let Target::Page(id) = &target else {
                continue;
            };

            if app.with_page(id, |page| fits(&page.kind, definition)) != Some(true) {
                continue;
            }
        }

        let Some(item) = display_item(app, target) else {
            continue;
        };

        let section = item.target.section();
        if sections[section].len() < LIMIT {
            sections[section].push(item);
        }
    }

    let page_fits =
        pending.is_none_or(|definition| PageKind::ALL.iter().any(|kind| fits(kind, definition)));

    if page_fits && search::scope(query, default) & PAGE != 0 {
        sections[2].push(new_page_item());
    }

    sections
}

fn query_parts(query: &str) -> Vec<(String, bool)> {
    let mut parts = Vec::new();
    let mut cursor = 0;

    while cursor < query.len() {
        let rest = &query[cursor..];

        if let Some(character) = rest.chars().next() {
            if character.is_whitespace() {
                let end = cursor + character.len_utf8();
                let mut whitespace_end = end;

                while whitespace_end < query.len() {
                    let Some(next) = query[whitespace_end..].chars().next() else {
                        break;
                    };

                    if !next.is_whitespace() {
                        break;
                    }

                    whitespace_end += next.len_utf8();
                }

                parts.push((query[cursor..whitespace_end].to_owned(), false));
                cursor = whitespace_end;
                continue;
            }
        }

        let start = cursor;
        let mut quote = None;
        let mut bracket_depth = 0usize;
        let mut escaped = false;

        for character in rest.chars() {
            if escaped {
                escaped = false;
                cursor += character.len_utf8();
                continue;
            }

            if character == '\\' {
                escaped = true;
                cursor += character.len_utf8();
                continue;
            }

            if let Some(current) = quote {
                if character == current {
                    quote = None;
                }

                cursor += character.len_utf8();
                continue;
            }

            match character {
                '"' | '\'' => quote = Some(character),
                '[' => bracket_depth += 1,
                ']' => bracket_depth = bracket_depth.saturating_sub(1),
                value if value.is_whitespace() && bracket_depth == 0 => break,
                _ => {}
            }

            cursor += character.len_utf8();
        }

        let token = &query[start..cursor];

        let operator_end = token.strip_prefix('!').and_then(|operator| {
            if let Some((name, _)) = operator.split_once(':') {
                let candidate = format!("!{}:", name);
                search::is_operator(&candidate).then_some(name.len() + 2)
            } else {
                search::is_operator(token).then_some(token.len())
            }
        });

        if let Some(end) = operator_end {
            parts.push((token[..end].to_owned(), true));

            if end < token.len() {
                parts.push((token[end..].to_owned(), false));
            }
        } else {
            parts.push((token.to_owned(), false));
        }
    }

    parts
}

fn navigation(sections: &[Vec<SearchItem>; 3], collapsed: [bool; 3]) -> Vec<Selection> {
    let mut items = Vec::new();

    for (category, results) in sections.iter().enumerate() {
        if results.is_empty() {
            continue;
        }

        items.push(Selection::Category(category));

        if !collapsed[category] {
            items.extend((0..results.len()).map(|result| Selection::Result(category, result)));
        }
    }

    items
}

#[component]
pub fn Search(scope: u8, pick: Callback<EntryId>) -> Element {
    let (app, client, mut ui, toast) = (
        use_context::<AppState>(),
        use_context::<Client>(),
        use_context::<UiState>(),
        use_toast(),
    );

    let index = use_memo(move || build_index(&app));
    let mut query = use_signal(String::new);
    let mut pending = use_signal(|| None::<EntryDefinition>);
    let mut collapsed = use_signal(|| [false; 3]);
    let mut active = use_signal(|| None::<Selection>);

    use_effect(move || {
        let _ = index.read();
        active.set(None);
    });

    let query_value = query();
    let pending_value = pending();

    let sections = build_sections(
        &app,
        &index.read(),
        &query_value,
        scope,
        pending_value.as_ref(),
    );

    let collapsed_value = collapsed();

    let titles = [
        tid!("search-entries"),
        tid!("search-new-entries"),
        tid!("search-pages"),
    ];

    let no_results = sections.iter().all(Vec::is_empty) && !query_value.trim().is_empty();

    let parts = query_parts(&query_value);
    let active_value = active();
    let key_sections = sections.clone();

    let activate = use_callback(move |target: Target| {
        let (client, toast) = (client.clone(), toast.clone());

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
                        query.set(String::new());
                        active.set(None);
                        return;
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
                        Err(error) => {
                            return toast.error(
                                tid!("toast-create-page-failed"),
                                ToastOptions::new().description(error).duration(ERR),
                            );
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
                Err(error) => toast.error(
                    tid!("toast-create-entry-failed"),
                    ToastOptions::new().description(error).duration(ERR),
                ),
            }
        });
    });

    let placeholder = match pending_value.as_ref() {
        Some(definition) => tid!("search-pick-page", kind: definition.kind.clone()),
        None => tid!("header-search-placeholder"),
    };

    let on_toggle = move |category: usize| {
        let mut values = collapsed();
        values[category] = !values[category];
        collapsed.set(values);
        active.set(Some(Selection::Category(category)));
    };

    let on_select = move |(category, item, target): (usize, usize, Target)| {
        active.set(Some(Selection::Result(category, item)));
        activate.call(target);
    };

    rsx! {
        document::Stylesheet {
            href: asset!("/assets/style/search/mod.css")
        }

        div {
            class: "search-overlay",
            onclick: move |_| ui.set_search(None),

            div {
                class: "search-box",
                onclick: move |event| event.stop_propagation(),

                div {
                    class: "search-head",

                    label {
                        class: "search-bar",

                        Icon { name: "search" }

                        div {
                            class: "search-input-scroll",

                            div {
                                class: "search-input-track",

                                span {
                                    class: "search-query-render",

                                    if query_value.is_empty() {
                                        span {
                                            class: "search-placeholder",
                                            "{placeholder}"
                                        }
                                    } else {
                                        for (index, (part, operator)) in parts.iter().enumerate() {
                                            span {
                                                key: "{index}",
                                                class: if *operator {
                                                    "search-query-operator"
                                                } else {
                                                    ""
                                                },
                                                "{part}"
                                            }
                                        }
                                    }
                                }

                                input {
                                    r#type: "text",
                                    class: "search-input",
                                    autocomplete: "off",
                                    spellcheck: "false",
                                    value: query_value,

                                    onmounted: move |event| async move {
                                        let _ = event.set_focus(true).await;
                                    },

                                    oninput: move |event| {
                                        query.set(event.value());
                                        active.set(None);
                                    },

                                    onkeydown: move |event: KeyboardEvent| {
                                        let nav = navigation(&key_sections, collapsed_value);

                                        match event.key() {
                                            Key::Tab => {
                                                if let Some(completed) =
                                                    search::complete_operator(&query())
                                                {
                                                    event.prevent_default();
                                                    query.set(completed);
                                                    active.set(None);
                                                }
                                            }

                                            Key::ArrowDown | Key::ArrowUp => {
                                                event.prevent_default();

                                                let current = active();
                                                let position = current.and_then(|selected| {
                                                    nav.iter().position(|item| *item == selected)
                                                });

                                                let next = match (event.key(), position) {
                                                    (Key::ArrowDown, Some(position)) => {
                                                        nav.get((position + 1) % nav.len()).copied()
                                                    }
                                                    (Key::ArrowUp, Some(position)) => {
                                                        nav.get(if position == 0 {
                                                            nav.len() - 1
                                                        } else {
                                                            position - 1
                                                        }).copied()
                                                    }
                                                    (Key::ArrowDown, None) => {
                                                        nav.iter()
                                                            .find(|item| matches!(
                                                                item,
                                                                Selection::Result(_, _)
                                                            ))
                                                            .copied()
                                                            .or_else(|| nav.first().copied())
                                                    }
                                                    (Key::ArrowUp, None) => {
                                                        nav.iter()
                                                            .rev()
                                                            .find(|item| matches!(
                                                                item,
                                                                Selection::Result(_, _)
                                                            ))
                                                            .copied()
                                                            .or_else(|| nav.last().copied())
                                                    }
                                                    _ => None,
                                                };

                                                active.set(next);
                                            }

                                            Key::ArrowLeft | Key::ArrowRight => {
                                                let category = match active() {
                                                    Some(Selection::Category(index))
                                                    | Some(Selection::Result(index, _)) => {
                                                        Some(index)
                                                    }
                                                    None => None,
                                                };

                                                if let Some(category) = category {
                                                    event.prevent_default();

                                                    let mut values = collapsed();
                                                    values[category] =
                                                        event.key() == Key::ArrowLeft;

                                                    collapsed.set(values);
                                                    active.set(Some(Selection::Category(category)));
                                                }
                                            }

                                            Key::Enter => {
                                                event.prevent_default();

                                                match active() {
                                                    Some(Selection::Result(category, item)) => {
                                                        if let Some(result) = key_sections
                                                            .get(category)
                                                            .and_then(|items| items.get(item))
                                                        {
                                                            activate.call(result.target.clone());
                                                        }
                                                    }

                                                    Some(Selection::Category(category)) => {
                                                        let mut values = collapsed();
                                                        values[category] = !values[category];
                                                        collapsed.set(values);
                                                        active.set(Some(
                                                            Selection::Category(category)
                                                        ));
                                                    }

                                                    None => {}
                                                }
                                            }

                                            Key::Escape => {
                                                event.prevent_default();
                                                ui.set_search(None);
                                            }

                                            _ => {}
                                        }
                                    },
                                }
                            }
                        }
                    }

                    if pending_value.is_none() {
                        div {
                            class: "search-help",

                            span {
                                b { "!entry" }
                                {titles[0].clone()}
                            }

                            span {
                                b { "!ne" }
                                {titles[1].clone()}
                            }

                            span {
                                b { "!page" }
                                {titles[2].clone()}
                            }

                            span {
                                b { "!id" }
                                " ID"
                            }

                            span {
                                b { "!kind:" }
                                {tid!("search-kind")}
                            }

                            span {
                                b { "!ext:" }
                                {tid!("search-extension")}
                            }

                            span {
                                b { "!chapter:" }
                                {tid!("search-chapter")}
                            }
                        }
                    }
                }

                div {
                    class: "search-list",

                    if no_results {
                        div {
                            class: "search-empty",
                            {tid!("search-empty")}
                        }
                    }

                    for category in 0..3 {
                        if !sections[category].is_empty() {
                            Category {
                                key: "{category}",
                                index: category,
                                title: titles[category].clone(),
                                items: sections[category].clone(),
                                collapsed: collapsed_value[category],
                                active: active_value
                                    == Some(Selection::Category(category)),
                                selected: match active_value {
                                    Some(Selection::Result(selected_category, item))
                                        if selected_category == category => Some(item),
                                    _ => None,
                                },
                                ontoggle: on_toggle,
                                onselect: on_select,
                            }
                        }
                    }
                }
            }
        }
    }
}
