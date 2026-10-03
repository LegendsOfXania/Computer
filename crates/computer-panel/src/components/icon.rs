use dioxus::prelude::*;
use include_dir::{include_dir, Dir};

static ICONS_DIR: Dir = include_dir!("crates/computer-panel/assets/icons");

#[component]
pub fn Icon(name: &'static str) -> Element {
    let content = ICONS_DIR
        .get_file(format!("{}.svg", name))
        .and_then(|f| f.contents_utf8())
        .unwrap_or("");

    rsx! {
        div { class: "icon", dangerous_inner_html: content }
    }
}