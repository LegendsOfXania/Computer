mod graph;

use dioxus::prelude::*;

use crate::{
    i18n::use_i18n,
    nav::use_nav_focus,
    state::{
        connection::ConnectionStatus,
        ui::UiState,
        AppState,
    },
};

#[component]
pub fn Editor() -> Element {
    let nav = use_nav_focus();
    let i18n = use_i18n();

    let status = use_context::<Signal<ConnectionStatus>>();
    let state = use_context::<AppState>();
    let ui = use_context::<UiState>();

    rsx! {
        document::Stylesheet { href: asset!("/assets/style/editor.css") }

        section {
            class: if nav.focused() { "editor focused" } else { "editor" },
            onclick: move |_| nav.focus(),

            match &*status.read() {
                ConnectionStatus::Connecting => rsx! {
                    div { class: "editor-status connecting",
                        div { class: "editor-status-header",
                            span { class: "editor-status-title", "{i18n.t(\"editor.title\")}" }
                            span { class: "editor-status-state", "{i18n.t(\"connection.connecting\")}" }
                        }
                        span { class: "editor-status-message",
                            "> {i18n.t(\"connection.connecting\")} "
                            span { class: "editor-status-dots" }
                        }
                    }
                },
                ConnectionStatus::Connected => rsx! {
                    if let Some(page_id) = *ui.opened_page.read() {
                        if let Some(page) = state.pages.read().get(&page_id) {
                            graph::Graph { entries: page.entries.clone(), links: false }
                        }
                    } else {
                        span { "{i18n.t(\"editor.no_page\")}" }
                    }
                },
                ConnectionStatus::Failed(error) => rsx! {
                    div { class: "editor-status failed",
                        div { class: "editor-status-header",
                            span { class: "editor-status-title", "{i18n.t(\"editor.title\")}" }
                            span { class: "editor-status-state", "{i18n.t(\"connection.failed\")}" }
                        }
                        span { class: "editor-status-message", "> {error}" }
                    }
                },
            }
        }
    }
}