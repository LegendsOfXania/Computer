
use dioxus::prelude::*;

use crate::components::utils::icon::Icon;

#[component]
pub fn Btn(
    #[props(default = "close")]
    icon: &'static str,
    #[props(into, default)]
    tooltip: String,
    #[props(into, default)]
    label: String,
    #[props(into, default)]
    class: String,
    #[props(default)]
    popover_target: Option<String>,
    #[props(default)]
    popover_action: Option<String>,
    #[props(default)]
    onclick: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        button {
            class: "btn {class}",
            r#type: "button",
            title: "{tooltip}",
            "aria-label": "{tooltip}",
            "popovertarget": popover_target,
            "popovertargetaction": popover_action,
            onclick,

            Icon { name: icon }

            if !label.is_empty() {
                span { class: "btn-label", "{label}" }
            }
        }
    }
}
