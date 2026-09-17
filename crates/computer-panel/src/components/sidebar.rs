use dioxus::prelude::*;

use crate::nav::use_nav;

#[component]
pub fn Sidebar() -> Element {
    let nav = use_nav();

    rsx! {
        document::Stylesheet { href: asset!("/assets/style/sidebar.css") }

        aside {
            class: if nav.focused() { "sidebar focused" } else { "sidebar" },

            onclick: move |_| nav.focus(),
        }
    }
}