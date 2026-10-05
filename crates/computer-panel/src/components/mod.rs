mod editor;
mod header;
mod btn;
mod icon;

use dioxus::prelude::*;

use crate::{components::{editor::Editor, header::Header}, i18n::use_i18n_root};

#[component]
pub fn Panel() -> Element {
    use_i18n_root();

    rsx! {
        document::Stylesheet { href: asset!("/assets/style/mod.css") }
        document::Stylesheet { href: asset!("/assets/style/icon.css") }
        document::Stylesheet { href: asset!("/assets/style/btn.css") }

        Header {}
        Editor {}
    }
}