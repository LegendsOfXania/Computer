use dioxus::prelude::*;

use crate::{
    nav::use_nav_focus, state::connection::ConnectionStatus,
};

#[component]
pub fn Editor(status: Signal<ConnectionStatus>) -> Element {
    let nav = use_nav_focus();
    
    rsx! {
        document::Stylesheet { href: asset!("/assets/style/editor.css") }

        section {
            class: if nav.focused() { "editor focused" } else { "editor" },

            onclick: move |_| nav.focus(),

            // TODO: editor
            "{status:?}"
        }
    }
}