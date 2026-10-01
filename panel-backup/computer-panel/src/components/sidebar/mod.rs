mod inspector;
mod library;

use dioxus::prelude::*;

use crate::{i18n::use_i18n, nav::use_nav};

use library::Library;

#[component]
pub fn Sidebar() -> Element {
    let nav = use_nav();
    let i18n = use_i18n();

    rsx! {
        document::Stylesheet { href: asset!("/assets/style/sidebar/mod.css") }

        aside {
            class: if nav.focused() { "sidebar focused" } else { "sidebar" },

            onclick: move |_| nav.focus(),

            div { class: "sidebar-header",
                span { class: "sidebar-header-title", {i18n.t("sidebar.library")} }
            }

            div { class: "sidebar-body",
                Library { nav }
            }
        }
    }
}