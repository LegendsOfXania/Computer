mod details;
mod tree;

use crate::{
    components::icon::Icon,
    i18n::I18n,
    state::{app::AppState, ui::UiState},
};

use computer_model::page::PageKind;
use details::Details;
use dioxus::{prelude::*, signals::ReadableExt};
use std::collections::HashSet;
use tree::{flatten, Row};

#[component]
pub fn Library() -> Element {
    let i18n = use_context::<I18n>();
    let app = use_context::<AppState>();
    let ui = use_context::<UiState>();

    let mut collapsed = use_signal(HashSet::<String>::new);
    let mut opened_page = ui.opened_page;
    let mut opened_entry = ui.opened_entry;

    let rows = flatten(app.pages.read().values());
    let opened = opened_page();
    let closed = collapsed.read().clone();

    let mut stack: Vec<(usize, bool)> = Vec::new();
    let mut visible = Vec::new();

    for row in rows {
        let depth = match &row {
            Row::Chapter { depth, .. } | Row::Page { depth, .. } => *depth,
        };

        while stack.last().is_some_and(|(d, _)| *d >= depth) {
            stack.pop();
        }

        if stack.iter().any(|(_, is_closed)| *is_closed) {
            continue;
        }

        if let Row::Chapter { path, .. } = &row {
            stack.push((depth, closed.contains(path)));
        }

        visible.push(row);
    }

    rsx! {
        document::Stylesheet { href: asset!("/assets/style/header/library.css") }

        div { class: "library",
            button { class: "library-trigger",
                Icon { name: "library" }
                span { {i18n.t("header.library")} }
            }

            div { class: "library-panel",
                section { class: "library-col library-list",
                    for row in visible {
                        match row {
                            Row::Chapter { name, path, depth } => rsx! {
                                p {
                                    key: "c{path}",
                                    class: "library-chapter",
                                    style: "--depth: {depth}",
                                    onclick: move |_| {
                                        collapsed.with_mut(|set| {
                                            if !set.remove(&path) {
                                                set.insert(path.clone());
                                            }
                                        });
                                    },
                                    Icon { name: "folder" }
                                    "{name}"
                                }
                            },
                            Row::Page { id, name, depth } => {
                                let icon = app
                                    .with_page(id, |page| match page.kind {
                                        PageKind::Sequence => "article",
                                        PageKind::Static => "bulletlist",
                                    })
                                    .unwrap_or("bulletlist");

                                rsx! {
                                    p {
                                        key: "p{id}",
                                        class: if opened == Some(id) {
                                            "library-page library-opened"
                                        } else {
                                            "library-page"
                                        },
                                        style: "--depth: {depth}",
                                        onclick: move |_| {
                                            if opened_page() != Some(id) {
                                                opened_page.set(Some(id));
                                                opened_entry.set(None);
                                            }
                                        },
                                        Icon { name: icon }
                                        "{name}"
                                    }
                                }
                            },
                        }
                    }
                }

                Details {}
            }
        }
    }
}
