use dioxus::prelude::*;

use crate::{components::icon::Icon, i18n::use_i18n};

#[component]
pub fn Header() -> Element {
    let i18n = use_i18n();

    rsx! {
        document::Stylesheet { href: asset!("/assets/style/header/mod.css") }

        header {
            nav { class: "nav",
                button {
                    span {
                        Icon { name: "library" }
                    }
                    span { {i18n.t("header.library")} }
                }
                button {
                    span {
                        Icon { name: "search" }
                    }
                    span { {i18n.t("header.search")} }
                }
            }

            div { class: "status",
                button { class: "push",
                    span { class: "push-text", {i18n.t("header.push")} }
                    span { class: "push-icon",
                        Icon { name: "chevron-right" }
                    }
                }
            }
        }
    }
}