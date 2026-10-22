use dioxus::prelude::*;

#[component]
pub fn Int(value: i32, onchange: EventHandler<i32>) -> Element {
    rsx! {
        input {
            class: "field-input",
            r#type: "number",
            value: "{value}",
            onchange: move |event| {
                if let Ok(v) = event.value().parse() {
                    onchange.call(v);
                }
            },
        }
    }
}
