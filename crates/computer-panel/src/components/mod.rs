mod header;
mod icon;

use dioxus::prelude::*;

use crate::{components::header::Header, i18n::use_i18n_root};

#[component]
pub fn Panel() -> Element {
    use_i18n_root();

    rsx! {
        document::Stylesheet { href: asset!("/assets/style/mod.css") }

        Header {}
    }
}