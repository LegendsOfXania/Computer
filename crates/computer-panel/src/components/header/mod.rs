use dioxus::prelude::*;

use crate::{components::icon::Icon, i18n::use_i18n};

#[component]
pub fn Header() -> Element {
    let i18n = use_i18n();
    let mut query = use_signal(String::new);

    rsx! {
        document::Stylesheet { href: asset!("/assets/style/header/mod.css") }
        document::Stylesheet { href: asset!("/assets/style/header/push.css") }
        document::Stylesheet { href: asset!("/assets/style/header/search.css") }

        header {
            nav { class: "nav",
                button {
                    span {
                        Icon { name: "library" }
                    }
                    span { {i18n.t("header.library")} }
                }

                label { class: "search",
                    span { class: "search-icon",
                        Icon { name: "search" }
                    }

                    span { class: "search-field",
                        span { class: "search-label", {i18n.t("header.search.name")} }

                        input {
                            class: "search-input",
                            r#type: "text",
                            required: true,
                            placeholder: i18n.t("header.search.placeholder"),
                            aria_label: i18n.t("header.search.name"),
                            autocomplete: "off",
                            spellcheck: "false",
                            value: query,
                            oninput: move |e| query.set(e.value()),
                        }
                    }
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