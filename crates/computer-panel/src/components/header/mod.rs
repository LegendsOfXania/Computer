mod library;

use crate::{
    components::{icon::Icon, toast::Toasts},
    ws::Client,
};

use dioxus::prelude::*;
use dioxus_i18n::tid;
use library::Library;

#[component]
pub fn Header() -> Element {
    let toasts = use_context::<Toasts>();
    let client = use_context::<Client>();

    let mut query = use_signal(String::new);

    rsx! {
        document::Stylesheet { href: asset!("/assets/style/header/mod.css") }
        document::Stylesheet { href: asset!("/assets/style/header/search.css") }
        document::Stylesheet { href: asset!("/assets/style/header/push.css") }

        header {
            div { class: "logo-icon",
                button { class: "logo-icon",
                    span { class: "logo-icon",
                        Icon { name: "computer" }
                    }
                }
            }

            nav { class: "nav",
                Library {}

                label { class: "search",
                    span { class: "search-icon",
                        Icon { name: "search" }
                    }

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
                        let (client, toasts) = (client.clone(), toasts.clone());
                        async move {
                            match client.push().await {
                                Ok(()) => toasts.push(1, tid!("toast-push-success"), String::new()),
                                Err(e) => toasts.push(3, tid!("toast-push-failed"), e),
                            }
                        }
                    },

                    span { class: "push-text", {tid!("header-push")} }
                    span { class: "push-icon",
                        Icon { name: "chevron-right" }
                    }
                }
            }
        }
    }
}