mod inspector;
mod library;

use dioxus::prelude::*;

use crate::{
    i18n::use_i18n,
    nav::use_nav,
    state::ui::UiState,
};

use inspector::Inspector;
use library::Library;

#[derive(Clone, Copy, PartialEq, Eq)]
enum SidebarView {
    Library,
    Inspector,
}

#[component]
pub fn Sidebar() -> Element {
    let mut nav = use_nav();
    let i18n = use_i18n();

    let ui = use_context::<UiState>();

    let mut view = use_signal(|| SidebarView::Library);

    use_effect(move || {
        nav.on_key(Callback::new(move |(key, _): (Key, Modifiers)| {
            if key != Key::Character("i".into()) || ui.opened_entry.read().is_none() {
                return false;
            }

            view.set(match view() {
                SidebarView::Library => SidebarView::Inspector,
                SidebarView::Inspector => SidebarView::Library,
            });

            true
        }));
    });

    let title = match view() {
        SidebarView::Library => i18n.t("sidebar.library"),
        SidebarView::Inspector => i18n.t("sidebar.inspector"),
    };

    rsx! {
        document::Stylesheet { href: asset!("/assets/style/sidebar/mod.css") }

        aside {
            class: if nav.focused() { "sidebar focused" } else { "sidebar" },

            onclick: move |_| nav.focus(),

            div { class: "sidebar-header",

                span { class: "sidebar-header-title", "{title}" }
            }

            div { class: "sidebar-body",

                match view() {
                    SidebarView::Library => rsx! {
                        Library { nav }
                    },

                    SidebarView::Inspector => rsx! {
                        Inspector { nav }
                    },
                }
            }
        }
    }
}