use dioxus::prelude::*;

#[component]
pub fn Toggle(value: bool, onchange: EventHandler<bool>) -> Element {
    rsx! {
        button {
            class: "field-toggle",
            onclick: move |_| onchange.call(!value),
            if value { "[x]" } else { "[ ]" }
        }
    }
}
