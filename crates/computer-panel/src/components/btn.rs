use dioxus::prelude::*;

use crate::components::icon::Icon;

#[component]
pub fn Btn(
    #[props(default = "close")] icon: &'static str,
    #[props(into, default)] tooltip: String,
    #[props(default)] onclick: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        button {
            class: "btn",
            title: "{tooltip}",
            onclick,
            Icon { name: icon }
        }
    }
}