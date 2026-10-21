use dioxus::prelude::*;

#[component]
pub fn Toggle(value: bool) -> Element {
    rsx! { span { "{value}" } }
}
