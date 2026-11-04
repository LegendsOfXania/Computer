use dioxus::prelude::*;
use dioxus_i18n::tid;

use crate::components::utils::{btn::Btn, icon::Icon};

#[component]
pub fn Card(
    #[props(default)] title: String,
    #[props(default)] duplicable: bool,
    #[props(default)] onduplicate: EventHandler<()>,
    #[props(default)] removable: bool,
    #[props(default)] onremove: EventHandler<()>,
    children: Element,
) -> Element {
    let mut open = use_signal(|| true);

    let headless = title.is_empty();

    rsx! {
        div { class: if headless { "card plain" } else if open() { "card open" } else { "card" },
            if !headless {
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

                        if removable {
                            Btn {
                                icon: "close",
                                tooltip: tid!("inspector-remove"),
                                onclick: move |_| onremove.call(()),
                            }
                        }
                    }
                }
            }

            if headless || open() {
                div { class: "card-body", {children} }
            }
        }
    }
}
