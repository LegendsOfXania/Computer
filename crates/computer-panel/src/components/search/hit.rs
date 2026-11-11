use dioxus::prelude::*;

use crate::components::utils::icon::Icon;

use super::SearchItem;

#[component]
pub(super) fn Hit(item: SearchItem, onclick: EventHandler<()>) -> Element {
    let style = if item.color.is_empty() {
        String::new()
    } else {
        format!("--entry-color: {};", item.color)
    };

    rsx! {
        button {
            class: "search-row",
            r#type: "button",
            onclick: move |_| onclick.call(()),
            span { class: "search-row-icon", style, Icon { name: item.icon.clone() } }
            span { class: "search-row-label", "{item.label}" }
            small { class: "search-row-hint", "{item.hint}" }
            small { class: "search-row-meta", "{item.meta}" }
        }
    }
}