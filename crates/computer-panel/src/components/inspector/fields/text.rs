use dioxus::prelude::*;

#[component]
pub fn Text(value: String) -> Element {
    rsx! { span { "{value}" } }
}
