use dioxus::prelude::*;

#[component]
pub fn IntField(
    value: i64,
    min: Option<i64>,
    max: Option<i64>,
    onchange: EventHandler<i64>,
) -> Element {
    rsx! {
        input {
            class: "field-input",
            r#type: "number",
            step: "1",
            min: min.map(|min| min.to_string()),
            max: max.map(|max| max.to_string()),
            value: "{value}",
            onchange: move |event: FormEvent| {
                if let Ok(number) = event.value().trim().parse::<i64>() {
                    let number = number
                        .max(min.unwrap_or(i64::MIN))
                        .min(max.unwrap_or(i64::MAX));

                    onchange.call(number);
                }
            },
        }
    }
}
