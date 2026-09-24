use std::collections::{BTreeMap, HashMap, HashSet};

use computer_model::page::{Page, PageKind};
use dioxus::prelude::*;

use crate::{
    nav::Nav, state::{AppState, ui::UiState},
};

#[derive(Clone, PartialEq, Default)]
struct Node {
    chapters: BTreeMap<String, Node>,
    pages: Vec<Page>,
}

impl Node {
    fn priority(&self) -> i32 {
        self.pages
            .iter()
            .map(|p| p.priority)
            .chain(self.chapters.values().map(Node::priority))
            .min()
            .unwrap_or(i32::MAX)
    }

    fn items(&self) -> Vec<Item> {
        let mut items: Vec<(i32, Item)> = self
            .pages
            .iter()
            .map(|p| (p.priority, Item::Page(p.clone())))
            .chain(
                self.chapters
                    .iter()
                    .map(|(name, node)| (node.priority(), Item::Chapter(name.clone(), node.clone()))),
            )
            .collect();

        items.sort_by_key(|(priority, _)| *priority);
        items.into_iter().map(|(_, item)| item).collect()
    }
}

enum Item {
    Page(Page),
    Chapter(String, Node),
}

#[derive(Clone, PartialEq)]
pub enum Row {
    Chapter {
        name: String,
        path: String,
        prefix: String,
        last: bool,
    },
    Page {
        page: Page,
        prefix: String,
        last: bool,
    },
}

pub fn flatten(
    pages: &HashMap<u64, Page>,
    expanded: &HashSet<String>,
) -> Vec<Row> {
    let mut root = Node::default();

    for page in pages.values() {
        let mut node = &mut root;

        if let Some(chapter) = &page.chapter {
            for part in chapter.split('.') {
                node = node.chapters.entry(part.to_string()).or_default();
            }
        }

        node.pages.push(page.clone());
    }

    let mut rows = Vec::new();
    flatten_into(&root, "", "", expanded, &mut rows);
    rows
}

fn flatten_into(
    node: &Node,
    indent: &str,
    path: &str,
    expanded: &HashSet<String>,
    rows: &mut Vec<Row>,
) {
    let items = node.items();
    let total = items.len();

    for (idx, item) in items.into_iter().enumerate() {
        let last = idx == total - 1;

        let connector = if last { "└── " } else { "├── " };
        let prefix = format!("{indent}{connector}");

        let child_indent = format!(
            "{indent}{}",
            if last { "    " } else { "│   " }
        );

        match item {
            Item::Page(page) => {
                rows.push(Row::Page {
                    page,
                    prefix,
                    last,
                });
            }

            Item::Chapter(name, child) => {
                let chapter_path = if path.is_empty() {
                    name.clone()
                } else {
                    format!("{path}.{name}")
                };

                rows.push(Row::Chapter {
                    name,
                    path: chapter_path.clone(),
                    prefix,
                    last,
                });

                if expanded.contains(&chapter_path) {
                    flatten_into(
                        &child,
                        &child_indent,
                        &chapter_path,
                        expanded,
                        rows,
                    );
                }
            }
        }
    }
}

#[component]
pub fn Library(mut nav: Nav) -> Element {
    let state = use_context::<AppState>();
    let mut ui = use_context::<UiState>();

    let mut expanded = use_signal(HashSet::<String>::new);
    let rows = use_memo(move || flatten(&state.pages.read(), &expanded()));
    let mut selected = use_signal(|| Option::<usize>::None);

    use_effect(move || {
        nav.on_key(Callback::new(move |(key, modifiers): (Key, Modifiers)| {
            let rows = rows();

            if rows.is_empty() {
                return false;
            }

            match key {
                Key::ArrowDown | Key::ArrowUp => {
                    let step = match key {
                        Key::ArrowDown => 1,
                        Key::ArrowUp => -1,
                        _ => unreachable!(),
                    };

                    let Some(selected_index) = selected() else {
                        selected.set(Some(0));
                        return true;
                    };

                    let start = selected_index as isize;
                    let mut index = start;

                    loop {
                        index = (index + step).rem_euclid(rows.len() as isize);

                        let is_chapter =
                            matches!(rows[index as usize], Row::Chapter { .. });

                        if !modifiers.shift() || is_chapter || index == start {
                            break;
                        }
                    }

                    selected.set(Some(index as usize));
                    true
                }

                Key::Enter => {
                    let Some(index) = selected() else {
                        return false;
                    };

                    match rows.get(index) {
                        Some(Row::Chapter { path, .. }) => {
                            let path = path.clone();

                            expanded.with_mut(|expanded| {
                                if !expanded.insert(path.clone()) {
                                    expanded.remove(&path);
                                }
                            });

                            true
                        }

                        Some(Row::Page { page, .. }) => {
                            ui.opened_page.set(Some(page.id));
                            true
                        }

                        None => false,
                    }
                }

                _ => false,
            }
        }));
    });

    rsx! {
        document::Stylesheet { href: asset!("/assets/style/sidebar/library.css") }

        div { class: "library",

            for (i , row) in rows().into_iter().enumerate() {
                {
                    let is_selected = nav.focused() && selected() == Some(i);
                    let pointer = if is_selected { "> " } else { "  " };

                    match row {

                        Row::Chapter { name, path, prefix, .. } => {
                            let is_expanded = expanded().contains(&path);
                            let icon = if is_expanded { "▼" } else { "▶" };
                            rsx! {
                                div {
                                    class: if is_selected { "library-row chapter selected" } else { "library-row chapter" },

                

                                    onclick: move |event| {
                                        event.stop_propagation();
                                        nav.focus();

                
                                        expanded
                
                                            .with_mut(|expanded| {
                                                if !expanded.insert(path.clone()) {
                                                    expanded.remove(&path);
                                                }
                                            });
                                        selected.set(Some(i));
                                    },
                                    span { class: "library-pointer", "{pointer}" }
                                    span { class: "library-tree-guide", "{prefix}" }
                                    span { class: "library-item-icon", "{icon} " }
                                    span { class: "library-chapter-name", "{name}/" }
                                }
                            }
                        }
                        Row::Page { page, prefix, .. } => {
                            let page_id = page.id;
                            let icon = match page.kind {
                                PageKind::Sequence => "[»]",
                                PageKind::Static => "[=]",
                            };
                            rsx! {
                                div {
                                    class: if is_selected { "library-row page selected" } else { "library-row page" },
                                    onclick: move |event| {
                                        event.stop_propagation();
                                        nav.focus();
                
                                        selected.set(Some(i));
                                        ui.opened_page.set(Some(page_id));
                                    },
                                    span { class: "library-pointer", "{pointer}" }
                                    span { class: "library-tree-guide", "{prefix}" }
                                    span { class: "library-item-icon", "{icon} " }
                                    span { class: "library-page-name", "{page.name}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}