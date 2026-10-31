use dioxus::prelude::*;
use dioxus_i18n::tid;

use crate::components::utils::icon::Icon;

#[component]
pub fn Collapse(
    title: String,
    #[props(default)] count: Option<usize>,
    #[props(default)] addable: bool,
    #[props(default)] onadd: EventHandler<()>,
    children: Element,
) -> Element {
    let mut open = use_signal(|| true);

    rsx! {
        div { class: if open() { "group open" } else { "group" },
            div { class: "group-head", onclick: move |_| open.set(!open()),
                Icon { name: "chevron-right-2" }

                if !title.is_empty() {
                    span { class: "group-title", "{title}" }
                }

                if let Some(count) = count {
                    span { class: "group-count", "({count})" }
                }

                if addable {
                    button {
                        class: "group-add",
                        title: tid!("inspector-add"),
                        onclick: move |event: MouseEvent| {
                            event.stop_propagation();
                            onadd.call(());
                        },
                        Icon { name: "plus" }
                    }
                }
            }

            if open() {
                div { class: "group-body", {children} }
            }
        }
    }
}
