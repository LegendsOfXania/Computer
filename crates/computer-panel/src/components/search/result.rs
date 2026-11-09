use dioxus::prelude::*;

use crate::components::utils::icon::Icon;

use super::SearchItem;

#[component]
pub(super) fn Result(item: SearchItem, active: bool, onclick: EventHandler<()>) -> Element {
    rsx! {
        button {
            class: if active { "search-row active" } else { "search-row" },
            r#type: "button",
            onclick: move |_| onclick.call(()),
            span { class: "search-row-icon", style: "--entry-color: {item.color};", Icon { name: item.icon.clone() } }
            span { class: "search-row-label", "{item.label}" }
            small { class: "search-row-hint", "{item.hint}" }
            small { class: "search-row-meta", "{item.meta}" }
        }
    }
}
