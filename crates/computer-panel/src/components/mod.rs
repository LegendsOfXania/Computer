mod header;
mod icon;

use dioxus::prelude::*;

use crate::components::header::Header;

#[component]
pub fn Panel() -> Element {
    rsx! {
        document::Stylesheet { href: asset!("/assets/style/mod.css") }

        Header {}
    }
}