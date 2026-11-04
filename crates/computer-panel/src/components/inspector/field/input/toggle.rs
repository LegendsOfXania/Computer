use dioxus::prelude::*;

#[component]
pub fn ToggleField(value: bool, onchange: EventHandler<bool>) -> Element {
    rsx! {
        input {
            class: "toggle",
            r#type: "checkbox",
            checked: value,
            onchange: move |event: FormEvent| onchange.call(event.checked()),
        }
    }
}
