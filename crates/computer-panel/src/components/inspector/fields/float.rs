use dioxus::prelude::*;

#[component]
pub fn FloatField(
    value: f64,
    min: Option<f64>,
    max: Option<f64>,
    onchange: EventHandler<f64>,
) -> Element {
    rsx! {
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
                        let number = number
                            .max(min.unwrap_or(f64::NEG_INFINITY))
                            .min(max.unwrap_or(f64::INFINITY));

                        onchange.call(number);
                    }
                }
            }
        }
    }
}
