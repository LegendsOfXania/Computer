mod library;

use crate::{
    components::{
        header::library::Library,
        utils::{
            icon::Icon,
            toast::{ERR, OK},
        },
    },
    ws::Client,
};
use dioxus::prelude::*;
use dioxus_i18n::tid;
use dioxus_primitives::toast::{ToastOptions, use_toast};

#[component]
pub fn Header() -> Element {
    let (toast, client) = (use_toast(), use_context::<Client>());
    let mut query = use_signal(String::new);

    rsx! {
        document::Stylesheet { href: asset!("/assets/style/header/mod.css") }
        document::Stylesheet { href: asset!("/assets/style/header/search.css") }
        document::Stylesheet { href: asset!("/assets/style/header/push.css") }

        header {
            button { class: "logo-icon", Icon { name: "computer" } }

            nav { class: "nav",
                Library {}

                label { class: "search",
                    span { class: "search-icon", Icon { name: "search" } }
                    span { class: "search-field",
                        span { class: "search-label", {tid!("header-search-name")} }
                        input {
                            class: "search-input",
                            r#type: "text",
                            required: true,
                            placeholder: tid!("header-search-placeholder"),
                            aria_label: tid!("header-search-name"),
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
                        let (client, toast) = (client.clone(), toast.clone());
                        async move {
                            match client.push().await {
                                Ok(()) => toast.success(tid!("toast-push-succes"), ToastOptions::new().duration(OK)),
                                Err(e) => toast.error(tid!("toast-push-failed"), ToastOptions::new().description(e).duration(ERR)),
                            }
                        }
                    },
                    span { class: "push-text", {tid!("header-push")} }
                    span { class: "push-icon", Icon { name: "chevron-right" } }
                }
            }
        }
    }
}
