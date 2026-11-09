use dioxus::prelude::*;

use crate::components::utils::icon::Icon;

use super::{SearchItem, Target, result::Result};

#[component]
pub(super) fn Category(
    index: usize,
    title: String,
    items: Vec<SearchItem>,
    collapsed: bool,
    active: bool,
    selected: Option<usize>,
    ontoggle: EventHandler<usize>,
    onselect: EventHandler<(usize, usize, Target)>,
) -> Element {
    rsx! {
        section { class: "search-category",
            button {
                class: if active { "search-category-head active" } else { "search-category-head" },
                r#type: "button",
                onclick: move |_| ontoggle.call(index),
                Icon { name: if collapsed { "chevron-right" } else { "chevron-down" } }
                span { class: "search-category-title", "{title}" }
                span { class: "search-category-count", "{items.len()}" }
            }

            if !collapsed {
                div { class: "search-category-items",
                    for (item_index, item) in items.iter().enumerate() {
                        Result {
                            key: "{index}-{item_index}",
                            item: item.clone(),
                            active: selected == Some(item_index),
                            onclick: {
                                let target = item.target.clone();
                                move |_| onselect.call((index, item_index, target.clone()))
                            },
                        }
                    }
                }
            }
        }
    }
}
