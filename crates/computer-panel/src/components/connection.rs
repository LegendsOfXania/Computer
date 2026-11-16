
use dioxus::prelude::*;
use dioxus_i18n::tid;

use crate::state::ConnectionStatus;

#[component]
pub fn Connection(status: ConnectionStatus) -> Element {
    let (title, message, attempt, reason, state_class) = match status {
        ConnectionStatus::Connecting => (
            tid!("connection-connecting-title"),
            tid!("connection-connecting-message"),
            None,
            None,
            "",
        ),

        ConnectionStatus::Reconnecting(attempt) => (
            tid!("connection-reconnecting-title"),
            tid!("connection-reconnecting-message"),
            Some(attempt),
            None,
            "connection-screen--reconnecting",
        ),

        ConnectionStatus::Failed(reason) => (
            tid!("connection-failed-title"),
            tid!("connection-failed-message"),
            None,
            Some(reason),
            "connection-screen--failed",
        ),

        _ => return rsx! {},
    };

    let attempt_label = tid!("connection-attempt");

    rsx! {
        main {
            class: "connection-screen {state_class}",

            div {
                class: "connection-scene",
                aria_hidden: "true",

                div { class: "connection-orbit connection-orbit--one" }
                div { class: "connection-orbit connection-orbit--two" }
                div { class: "connection-orbit connection-orbit--three" }

                div { class: "connection-star connection-star--one" }
                div { class: "connection-star connection-star--two" }
                div { class: "connection-star connection-star--three" }
                div { class: "connection-star connection-star--four" }
                div { class: "connection-star connection-star--five" }

                div { class: "connection-planet connection-planet--main" }
                div { class: "connection-planet connection-planet--purple" }
                div { class: "connection-planet connection-planet--teal" }
                div { class: "connection-planet connection-planet--blue" }
                div { class: "connection-planet connection-planet--small" }
            }

            div {
                class: "connection-status",

                if reason.is_some() {
                    span {
                        class: "connection-indicator connection-indicator-error"
                    }
                } else {
                    span {
                        class: "connection-indicator"
                    }
                }

                div {
                    class: "connection-content",

                    h1 { "{title}" }
                    p { "{message}" }

                    if let Some(attempt) = attempt {
                        span {
                            class: "connection-attempt",
                            "{attempt_label} {attempt}"
                        }
                    }

                    if let Some(reason) = reason {
                        p {
                            class: "connection-reason",
                            "{reason}"
                        }
                    }
                }
            }
        }
    }
}
