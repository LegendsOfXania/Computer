mod library;

use dioxus::prelude::*;
use dioxus_i18n::tid;

use crate::{
    actions::Actions,
    components::{header::library::Library, utils::icon::Icon},
    search::{ENTRY, NEW_ENTRY, PAGE},
    state::{
        app::AppState,
        ui::{Picked, SearchRequest, UiState},
    },
};

#[component]
pub fn Header() -> Element {
    let (app, mut ui, actions) = (
        use_context::<AppState>(),
        use_context::<UiState>(),
        use_context::<Actions>(),
    );

    let open = use_callback(move |picked: Picked| match picked {
        Picked::Entry(id) => ui.set_opened_entry(&app, Some(id)),
        Picked::Page(id) => ui.set_opened_page(Some(id)),
    });

    let search = SearchRequest {
        scope: PAGE | ENTRY | NEW_ENTRY,
        accepts: None,
        pick: open,
    };

    rsx! {

        header {
            button { class: "logo-icon", Icon { name: "computer" } }

            nav { class: "nav",
                Library {}

                button { onclick: move |_| ui.set_search(Some(search.clone())),
                    Icon { name: "search" }
                    span { {tid!("header-search-name")} }
                }
            }

            div { class: "status",
                button { class: "push", onclick: move |_| actions.push(),
                    span { class: "push-text", {tid!("header-push")} }
                    span { class: "push-icon", Icon { name: "chevron-right" } }
                }
            }
        }
    }
}
