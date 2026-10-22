use dioxus::prelude::*;

#[component]
pub fn Text(value: String, onchange: EventHandler<String>) -> Element {
    rsx! {
        input {
            class: "field-input",
            value: "{value}",
            onchange: move |event| onchange.call(event.value()),
        }
    }
}
