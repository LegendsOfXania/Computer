use dioxus::prelude::*;

use crate::components::utils::icon::Icon;

#[component]
pub fn IntField(
    #[props(default = "binary")] icon: &'static str,
    value: i64,
    min: Option<i64>,
    max: Option<i64>,
    onchange: EventHandler<i64>,
) -> Element {
    rsx! {
        div { class: "field-icon-wrap",
            Icon { name: icon }

            input {
                class: "field-input",
                r#type: "number",
                step: "1",
                min: min.map(|min| min.to_string()),
                max: max.map(|max| max.to_string()),
                value: "{value}",
                onchange: move |event: FormEvent| {
                    if let Ok(number) = event.value().trim().parse::<i64>() {
                        onchange.call(
                            number
                            .max(min.unwrap_or(i64::MIN))
                            .min(max.unwrap_or(i64::MAX)),
                        );
                    }
                },
            }
        }
    }
}

#[component]
pub fn FloatField(
    #[props(default = "binary")] icon: &'static str,
    value: f64,
    min: Option<f64>,
    max: Option<f64>,
    onchange: EventHandler<f64>,
) -> Element {
    rsx! {
        div { class: "field-icon-wrap",
            Icon { name: icon }

            input {
                class: "field-input",
                r#type: "number",
                step: "any",
                min: min.map(|min| min.to_string()),
                max: max.map(|max| max.to_string()),
                value: "{value}",
                onchange: move |event: FormEvent| {
                    if let Ok(number) = event.value().trim().parse::<f64>() {
                        if number.is_finite() {
                            onchange.call(
                                number
                                    .max(min.unwrap_or(f64::MIN))
                                    .min(max.unwrap_or(f64::MAX)),
                            );
                        }
                    }
                },
            }
        }
    }
}
