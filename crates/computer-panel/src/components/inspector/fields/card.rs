use dioxus::prelude::*;
use dioxus_i18n::tid;

use crate::components::utils::{btn::Btn, icon::Icon};

#[component]
pub fn Card(
    title: String,
    #[props(default)] duplicable: bool,
    #[props(default)] onduplicate: EventHandler<()>,
    onremove: EventHandler<()>,
    children: Element,
) -> Element {
    let mut open = use_signal(|| true);

    rsx! {
        div { class: if open() { "card open" } else { "card" },
            div { class: "card-head", onclick: move |_| open.set(!open()),
                Icon { name: "chevron-right-2" }
                span { class: "card-title", "{title}" }

                div { class: "card-actions",
                    if duplicable {
                        Btn {
                            icon: "plus-box",
                            tooltip: tid!("inspector-duplicate"),
                            onclick: move |_| onduplicate.call(()),
                        }
                    }

                    Btn {
                        icon: "close",
                        tooltip: tid!("inspector-remove"),
                        onclick: move |_| onremove.call(()),
                    }
                }
            }

            if open() {
                div { class: "card-body", {children} }
            }
        }
    }
}
