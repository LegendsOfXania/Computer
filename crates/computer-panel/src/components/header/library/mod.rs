mod details;
mod tree;

use std::collections::HashSet;

use dioxus::prelude::*;
use dioxus_i18n::tid;

use crate::{
    components::utils::icon::Icon,
    state::{app::AppState, ui::UiState},
};
use details::Details;
use tree::{Row, flatten};

#[component]
pub fn Library() -> Element {
    let (app, mut ui) = (use_context::<AppState>(), use_context::<UiState>());
    let mut collapsed = use_signal(HashSet::<String>::new);

    let rows = use_memo(move || flatten(app.library.read().pages.values()));

    let (opened, closed) = (ui.get_opened_page(), collapsed.read().clone());

    let mut stack = Vec::new();

    let visible: Vec<_> = rows
        .read()
        .iter()
        .filter(|row| {
            let (Row::Chapter { depth, .. } | Row::Page { depth, .. }) = row;

            while stack.last().is_some_and(|&(last, _)| last >= *depth) {
                stack.pop();
            }

            if stack.iter().any(|&(_, is_closed)| is_closed) {
                return false;
            }

            if let Row::Chapter { path, .. } = row {
                stack.push((*depth, closed.contains(path)));
            }

            true
        })
        .map(|row| match row {
            Row::Chapter { name, path, depth } => (None, name.clone(), path.clone(), "", *depth),
            Row::Page {
                id,
                name,
                icon,
                depth,
            } => (Some(*id), name.clone(), id.to_string(), *icon, *depth),
        })
        .collect();

    rsx! {

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

                    for (page , name , path , icon , depth) in visible {
                        match page {
                            None => rsx! {
                                p {
                                    key: "c{path}",
                                    class: "library-chapter",
                                    style: "--depth: {depth}",
                                    onclick: move |_| {
                                        collapsed
                                            .with_mut(|set| {
                                                if !set.remove(&path) {
                                                    set.insert(path.clone());
                                                }
                                            });
                                    },
                                    Icon { name: "folder" }
                                    "{name}"
                                }
                            },

                            Some(id) => rsx! {
                                p {
                                    key: "p{path}",
                                    class: if opened == Some(id) { "library-page library-opened" } else { "library-page" },
                                    style: "--depth: {depth}",
                                    onclick: move |_| {
                                        if ui.get_opened_page() != Some(id) {
                                            ui.set_opened_page(Some(id));
                                        }
                                    },
                                    Icon { name: icon }
                                    "{name}"
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
