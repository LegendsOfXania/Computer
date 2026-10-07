mod library;

use dioxus::prelude::*;

use crate::{components::{btn::Btn, icon::Icon}, i18n::use_i18n};

#[component]
pub fn Header() -> Element {
    let i18n = use_i18n();
    let (name, id, kind, priority, chapter) = ("Intro", "01010100101", "sequence", "0", "chap1.chap2.chap3");
    let mut query = use_signal(String::new);

    rsx! {
        document::Stylesheet { href: asset!("/assets/style/header/mod.css") }
        document::Stylesheet { href: asset!("/assets/style/header/library.css") }
        document::Stylesheet { href: asset!("/assets/style/header/search.css") }
        document::Stylesheet { href: asset!("/assets/style/header/push.css") }

        header {
            nav { class: "nav",

                div { class: "library",
                    button { class: "library-trigger",
                        Icon { name: "library" }
                        span { {i18n.t("header.library")} }
                    }

                    div { class: "library-panel",
                        section { class: "library-col" }
                        section { class: "library-col library-details",
                            div { class: "library-head",
                                p { class: "library-name",
                                    Icon { name: "book-open" }
                                    {name}
                                }
                                p { class: "library-id", {id} }
                            }

                            div { class: "library-meta",
                                p {
                                    span { class: "library-label",
                                        Icon { name: "git-commit" }
                                        {i18n.t("header.library.opened_page.kind")}
                                    }
                                    span { "{kind}" }
                                }
                                p {
                                    span { class: "library-label",
                                        Icon { name: "arrow-big-up-dash" }
                                        {i18n.t("header.library.opened_page.priority")}
                                    }
                                    span { "{priority}" }
                                }
                                p {
                                    span { class: "library-label",
                                        Icon { name: "bookmark" }
                                        {i18n.t("header.library.opened_page.chapter")}
                                    }
                                    span { "{chapter}" }
                                }
                            }

                            div { class: "library-footer",
                                Btn {
                                    icon: "plus",
                                    tooltip: Some(i18n.t("header.library.tooltip.create_page")),
                                }
                            }
                        }
                    }
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