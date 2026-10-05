use dioxus::prelude::*;

use crate::components::icon::Icon;

#[component]
pub fn Btn(icon: &'static str, tooltip: Option<String>) -> Element {
    let mut is_hovered = use_signal(|| false);

    rsx! {
        button {
            class: "btn",
            class: if is_hovered() { "btn-hovered" } else { "btn-unselected" },
            onmouseenter: move |_| is_hovered.set(true),
            onmouseleave: move |_| is_hovered.set(false),
            title: tooltip.unwrap_or_default(),
            Icon { name: icon }
        }
    }
}