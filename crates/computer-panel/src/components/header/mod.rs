use dioxus::prelude::*;

use crate::components::icon::Icon;

#[component]
pub fn Header() -> Element {
    rsx! {
        document::Stylesheet { href: asset!("/assets/style/header/mod.css") }

        header {
            nav { class: "nav",
                button { "Library" }
                button { "Search" }
            }

            div { class: "status",
                button { class: "push",
                    span { class: "push-text", "Push" }
                    span { class: "push-icon",
                        Icon { name: "chevron-right" }
                    }
                }
            }
        }
    }
}