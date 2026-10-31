use dioxus::prelude::*;

use crate::components::utils::icon::Icon;

#[component]
pub fn TextField(value: String, multiline: bool, onchange: EventHandler<String>) -> Element {
    rsx! {
        div { class: "field-text-wrap",
            Icon { name: "pen-square" }

            textarea { class: if multiline { "field-input field-text multiline" } else { "field-input field-text" },
                rows: 1,
                spellcheck: false,
                value: "{value}",

                onkeydown: move |event: KeyboardEvent| {
                    if !multiline && event.key() == Key::Enter {
                        event.prevent_default();
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
}
