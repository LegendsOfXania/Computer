use dioxus::prelude::*;

#[component]
pub fn Float(value: f64, onchange: EventHandler<f64>) -> Element {
    rsx! {
        input {
            class: "field-input",
            r#type: "number",
            value: "{value}",
            step: "any",
            onchange: move |event| {
                if let Ok(v) = event.value().parse() {
                    onchange.call(v);
                }
            },
        }
    }
}
