use dioxus::prelude::*;
use dioxus_i18n::tid;

use crate::state::ConnectionStatus;

#[component]
pub fn Connection(status: ConnectionStatus) -> Element {
    let (title, message, attempt, reason) = match status {
        ConnectionStatus::Connecting => (
            tid!("connection-connecting-title"),
            tid!("connection-connecting-message"),
            None,
            None,
        ),

        ConnectionStatus::Reconnecting(attempt) => (
            tid!("connection-reconnecting-title"),
            tid!("connection-reconnecting-message"),
            Some(attempt),
            None,
        ),

        ConnectionStatus::Failed(reason) => (
            tid!("connection-failed-title"),
            tid!("connection-failed-message"),
            None,
            Some(reason),
        ),

        _ => return rsx! {},
    };

    let attempt_label = tid!("connection-attempt");

    rsx! {
        main {
            class: "connection-screen",

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