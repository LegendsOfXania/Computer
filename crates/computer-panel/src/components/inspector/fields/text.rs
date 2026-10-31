use dioxus::prelude::*;

#[component]
pub fn TextField(value: String, multiline: bool, onchange: EventHandler<String>) -> Element {
    rsx! {
        textarea { class: if multiline { "field-input field-text multiline" } else { "field-input field-text" },
            rows: 1,
            spellcheck: false,
            value: "{value}",

            onkeydown: move |event: KeyboardEvent| {
                if !multiline && event.key() == Key::Enter {
                    event.prevent_default();
                    let _ = document::eval("document.activeElement?.blur()");
                }
            },

            onchange: move |event: FormEvent| {
                let text = event.value();

                onchange.call(if multiline {
                    text
                } else {
                    text.replace(['\r', '\n'], " ")
                })
            }
        }
    }
}
