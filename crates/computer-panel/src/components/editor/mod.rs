mod graph;
mod layout;
mod node;
mod status;

use dioxus::prelude::*;

use crate::{
    i18n::use_i18n, nav::use_nav_focus, state::{
        AppState, connection::ConnectionStatus, ui::UiState,
    },
};

#[component]
pub fn Editor() -> Element {
    let nav = use_nav_focus();
    let i18n = use_i18n();

    let ui = use_context::<UiState>();
    let state = use_context::<AppState>();
    let status: Signal<ConnectionStatus> = use_context::<Signal<ConnectionStatus>>();

    use_effect(move || {
        if ui.opened_page.read().is_some() {
            nav.focus();
        }
    });

    rsx! {
        document::Stylesheet { href: asset!("/assets/style/editor/mod.css") }

        section {
            class: if nav.focused() { "editor focused" } else { "editor" },
            onclick: move |_| nav.focus(),
            match &*status.read() {
                ConnectionStatus::Connected => rsx! {
                    if let Some(page_id) = *ui.opened_page.read() {
                        if let Some(page) = state.pages.read().get(&page_id) {
                            graph::Graph {
                                key: "{page_id}",
                                entries: page.entries.clone(),
                                kind: page.kind,
                                nav,
                            }
                        }
                    } else {
                        span { "{i18n.t(\"editor.no_page\")}" }
                    }
                },
                _ => rsx! {
                    status::Status {}
                },
            }
        }
    }
}