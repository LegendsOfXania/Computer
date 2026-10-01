use dioxus::prelude::*;

use crate::{
    components::{editor::Editor, sidebar::Sidebar},
    i18n::use_i18n_root,
    nav::{use_nav_root, Navigation},
    state::status::ConnectionStatus,
};

#[component]
pub fn Shell() -> Element {
    use_nav_root();
    use_i18n_root();

    let status = use_context::<Signal<ConnectionStatus>>();
    let nav = use_context::<Navigation>();

    rsx! {
        document::Stylesheet { href: asset!("/assets/style/shell.css") }

        div {
            class: "shell",
            tabindex: "0",
            autofocus: true,

            onkeydown: move |event| {
                match event.key() {
                    Key::Tab if event.modifiers().shift() => {
                        nav.previous();
                        event.prevent_default();
                    }

                    Key::Tab => {
                        nav.next();
                        event.prevent_default();
                    }

                    Key::Escape => {
                        // TODO: parent
                        event.prevent_default();
                    }

                    key => {
                        if nav.dispatch(key, event.modifiers()) {
                            event.prevent_default();
                        }
                    }
                }
            },

            Editor {}

            if matches!(&*status.read(), ConnectionStatus::Connected) {
                Sidebar {}
            }
        }
    }
}