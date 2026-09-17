use dioxus::prelude::*;

use crate::{
    components::{
        editor::Editor,
        sidebar::Sidebar,
    },
    nav::{
        use_nav_root,
        Navigation,
    },
    state::connection::ConnectionStatus,
};

#[component]
pub fn Shell(status: Signal<ConnectionStatus>) -> Element {
    use_nav_root();

    let navigation = use_context::<Navigation>();

    rsx! {
        document::Stylesheet { href: asset!("/assets/style/shell.css") }

        div {
            class: "shell",
            tabindex: "0",
            autofocus: true,

            onkeydown: move |event| {
                match event.key() {
                    Key::Tab if event.modifiers().shift() => {
                        navigation.previous();
                        event.prevent_default();
                    }

                    Key::Tab => {
                        navigation.next();
                        event.prevent_default();
                    }

                    Key::Escape => {
                        event.prevent_default();
                    }

                    _ => {}
                }
            },

            Editor { status }
            Sidebar {}
        }
    }
}