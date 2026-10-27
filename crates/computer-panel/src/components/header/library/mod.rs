mod details;
mod tree;

use crate::{
    components::utils::icon::Icon,
    state::{app::AppState, ui::UiState},
};
use computer_model::{id::Identifier, page::PageKind};
use details::Details;
use dioxus::{prelude::*, signals::ReadableExt};
use dioxus_i18n::tid;
use std::collections::HashSet;
use tree::{Row, flatten};

#[component]
pub fn Library() -> Element {
    let (app, ui) = (use_context::<AppState>(), use_context::<UiState>());
    let mut collapsed = use_signal(HashSet::<String>::new);
    let mut opened_page = ui.page;

    let (rows, opened, closed) = (
        flatten(app.pages.read().values()),
        opened_page(),
        collapsed.read().clone(),
    );

    let mut stack = Vec::new();
    let visible: Vec<_> = rows
        .into_iter()
        .filter_map(|row| {
            let depth = match &row {
                Row::Chapter { depth, .. } | Row::Page { depth, .. } => *depth,
            };

            while stack.last().is_some_and(|&(d, _)| d >= depth) {
                stack.pop();
            }

            if stack.iter().any(|&(_, is_closed)| is_closed) {
                return None;
            }

            if let Row::Chapter { path, .. } = &row {
                stack.push((depth, closed.contains(path)));
            }

            Some(row)
        })
        .collect();

    rsx! {
        document::Stylesheet { href: asset!("/assets/style/header/library.css") }

        div { class: "library",
            button { class: "library-trigger",
                Icon { name: "library" }
                span { {tid!("header-library")} }
            }

            div { class: "library-panel",
                section { class: "library-col library-list",
                    if visible.is_empty() {
                        p { class: "library-empty", {tid!("library-empty")} }
                    }

                    for row in visible {
                        match row {
                            Row::Chapter { name, path, depth } => {
                                let key = path.clone();
                                rsx! {
                                    p {
                                        key: "c{path}",
                                        class: "library-chapter",
                                        style: "--depth: {depth}",
                                        onclick: move |_| {
                                            collapsed.with_mut(|set| {
                                                if !set.remove(&key) {
                                                    set.insert(key.clone());
                                                }
                                            });
                                        },
                                        Icon { name: "folder" }
                                        "{name}"
                                    }
                                }
                            },

                            Row::Page { id, name, depth } => {
                                let icon = app
                                    .with_page(&id, |page| match page.kind {
                                        PageKind::Sequence => "article",
                                        PageKind::Static => "bulletlist",
                                    })
                                    .unwrap_or("bulletlist");

                                let selected = opened.as_ref() == Some(&id);
                                let key = id.clone().into_string();
                                let target = id.clone();

                                rsx! {
                                    p {
                                        key: "p{key}",
                                        class: if selected {
                                            "library-page library-opened"
                                        } else {
                                            "library-page"
                                        },
                                        style: "--depth: {depth}",
                                        onclick: move |_| {
                                            if opened_page().as_ref() != Some(&target) {
                                                opened_page.set(Some(target.clone()));
                                            }
                                        },
                                        Icon { name: icon }
                                        "{name}"
                                    }
                                }
                            }
                        }
                    }
                }

                Details {}
            }
        }
    }
}
