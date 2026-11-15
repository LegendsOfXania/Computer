use std::{fmt::Display, str::FromStr};

use dioxus::prelude::*;

use crate::components::utils::icon::Icon;

pub trait Number: Copy + PartialOrd + Display + FromStr + 'static {
    const STEP: &'static str;
    const LOW: Self;
    const HIGH: Self;

    fn is_valid(self) -> bool;
}

impl Number for i64 {
    const STEP: &'static str = "1";
    const LOW: Self = Self::MIN;
    const HIGH: Self = Self::MAX;

    fn is_valid(self) -> bool {
        true
    }
}

impl Number for f64 {
    const STEP: &'static str = "any";
    const LOW: Self = Self::MIN;
    const HIGH: Self = Self::MAX;

    fn is_valid(self) -> bool {
        self.is_finite()
    }
}

fn bounded<T: Number>(value: T, min: T, max: T) -> T {
    let value = if value < min { min } else { value };

    if value > max { max } else { value }
}

#[component]
pub fn NumberField<T: Number>(
    #[props(default = "binary")] icon: &'static str,
    value: T,
    min: Option<T>,
    max: Option<T>,
    onchange: EventHandler<T>,
) -> Element {
    rsx! {
        div { class: "field-icon-wrap",
            Icon { name: icon }

            input {
                class: "field-input",
                r#type: "number",
                step: T::STEP,
                min: min.map(|min| min.to_string()),
                max: max.map(|max| max.to_string()),
                value: "{value}",
                onchange: move |event: FormEvent| {
                    if let Ok(number) = event.value().trim().parse::<T>()
                        && number.is_valid()
                    {
                        onchange
                            .call(bounded(number, min.unwrap_or(T::LOW), max.unwrap_or(T::HIGH)));
                    }
                },
            }
        }
    }
}
