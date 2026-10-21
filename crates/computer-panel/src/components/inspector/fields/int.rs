use dioxus::prelude::*;

#[component]
pub fn Int(value: i32) -> Element {
    rsx! { span { "{value}" } }
}
