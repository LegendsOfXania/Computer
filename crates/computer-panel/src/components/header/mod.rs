mod library;

use crate::{
    components::{
        header::library::Library,
        utils::{
            icon::Icon,
            toast::{ERR, OK},
        },
    },
    search::{ENTRY, PAGE},
    state::{app::AppState, ui::UiState},
    ws::Client,
};
use computer_model::id::EntryId;
use dioxus::prelude::*;
use dioxus_i18n::tid;
use dioxus_primitives::toast::{ToastOptions, use_toast};

#[component]
pub fn Header() -> Element {
    let (app, mut ui, toast, client) = (
        use_context::<AppState>(),
        use_context::<UiState>(),
        use_toast(),
        use_context::<Client>(),
    );

    let open = use_callback(move |id: EntryId| ui.set_opened_entry(&app, Some(id)));

    rsx! {
        document::Stylesheet { href: asset!("/assets/style/header/mod.css") }
        document::Stylesheet { href: asset!("/assets/style/header/push.css") }

        header {
            button { class: "logo-icon", Icon { name: "computer" } }

            nav { class: "nav",
                Library {}

                button { onclick: move |_| ui.set_search(Some((PAGE | ENTRY, open))),
                    Icon { name: "search" }
                    span { {tid!("header-search-name")} }
                }
            }

            div { class: "status",
                button {
                    class: "push",
                    onclick: move |_| {
                        let (client, toast) = (client.clone(), toast);
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
