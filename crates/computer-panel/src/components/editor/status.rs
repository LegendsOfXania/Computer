use dioxus::prelude::*;

use crate::{i18n::use_i18n, state::status::ConnectionStatus};

#[component]
pub fn Status() -> Element {
    let i18n = use_i18n();
    let status = use_context::<Signal<ConnectionStatus>>();

    let (class, state, message) = match &*status.read() {
        ConnectionStatus::Connecting => (
            "connecting",
            i18n.t("connection.connecting"),
            format!("> {} ", i18n.t("connection.connecting")),
        ),
        ConnectionStatus::Failed(error) => (
            "failed",
            i18n.t("connection.failed"),
            format!("> {error}"),
        ),
        _ => return rsx! {},
    };

    rsx! {
        document::Stylesheet { href: asset!("/assets/style/editor/status.css") }

        div { class: "editor-status {class}",
            div { class: "editor-status-header",
                span { class: "editor-status-title", "{i18n.t(\"editor.title\")}" }
                span { class: "editor-status-state", "{state}" }
            }

            span { class: "editor-status-message",
                "{message}"
                if matches!(&*status.read(), ConnectionStatus::Connecting) {
                    span { class: "editor-status-dots" }
                }
            }
        }
    }
}