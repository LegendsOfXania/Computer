use dioxus::prelude::*;

use crate::{nav::Nav, state::ui::UiState};

#[component]
pub fn Inspector(mut nav: Nav) -> Element {
    let ui = use_context::<UiState>();

    let Some(entry) = ui.opened_entry.read().as_ref().copied() else {
        return rsx! {};
    };

    rsx! {
        document::Stylesheet { href: asset!("/assets/style/sidebar/inspector.css") }

        div { class: "inspector", "{entry:?}" }
    }
}