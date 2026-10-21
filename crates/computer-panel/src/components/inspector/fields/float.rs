use dioxus::prelude::*;

#[component]
pub fn Float(value: f64) -> Element {
    rsx! { span { "{value}" } }
}
