mod library;
mod tree;

use crate::{
    components::{icon::Icon, toast::Toasts},
    i18n::I18n,
    ws::Client,
};

use computer_model::protocol::request::Request;
use dioxus::prelude::*;
use library::Library;

#[component]
pub fn Header() -> Element {
    let i18n = use_context::<I18n>();
    let toasts = use_context::<Toasts>();
    let client = use_context::<Client>();

    let mut query = use_signal(String::new);

    rsx! {
        document::Stylesheet { href: asset!("/assets/style/header/mod.css") }
        document::Stylesheet { href: asset!("/assets/style/header/search.css") }
        document::Stylesheet { href: asset!("/assets/style/header/push.css") }

        header {
            nav { class: "nav",
                Library {}

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
                button {
                    class: "push",
                    onclick: move |_| {
                        let (client, toasts, i18n) = (client.clone(), toasts.clone(), i18n.clone());
                        async move {
                            match client.request(Request::Push).await {
                                Ok(()) => toasts.push(1, i18n.t("toast.push.success"), String::new()),
                                Err(e) => toasts.push(3, i18n.t("toast.push.failed"), e),
                            }
                        }
                    },

                    span { class: "push-text", {i18n.t("header.push")} }
                    span { class: "push-icon",
                        Icon { name: "chevron-right" }
                    }
                }
            }
        }
    }
}