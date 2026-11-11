use dioxus::prelude::*;

use crate::components::utils::icon::Icon;

use super::{SearchItem, hit::Hit};

#[component]
pub(super) fn Category(
    index: usize,
    title: String,
    items: Vec<SearchItem>,
    collapsed: bool,
    active: bool,
    selected: Option<usize>,
    ontoggle: EventHandler<usize>,
    onselect: EventHandler<(usize, usize)>,
) -> Element {
    rsx! {
        section { class: "search-category",
            button {
                class: if active { "search-category-head active" } else { "search-category-head" },
                r#type: "button",
                onclick: move |_| ontoggle.call(index),
                Icon { name: if collapsed { "chevron-right" } else { "chevron-down" } }
                span { class: "search-category-title", "{title}" }
            }

            if !collapsed {
                div { class: "search-category-items",
                    for (item_index, item) in items.into_iter().enumerate() {
                        Hit {
                            key: "{item_index}",
                            item,
                            onclick: move |_| onselect.call((index, item_index)),
                        }
                    }
                }
            }
        }
    }
}
