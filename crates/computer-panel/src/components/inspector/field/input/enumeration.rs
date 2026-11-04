use dioxus::prelude::*;

use crate::components::utils::icon::Icon;

#[component]
pub fn EnumField(
    #[props(default = "bulletlist")] icon: &'static str,
    options: Vec<String>,
    value: String,
    onchange: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "field-icon-wrap",
            Icon { name: icon }

            select {
                class: "field-input",
                onchange: move |event: FormEvent| onchange.call(event.value()),
                for option in options.iter() {
                    option {
                        key: "{option}",
                        value: "{option}",
                        selected: *option == value,
                        "{option}"
                    }
                }
            }
        }
    }
}
