use std::time::Duration;

use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;

use crate::components::utils::icon::Icon;

const OK: Duration = Duration::from_secs(5);
const WARN: Duration = Duration::from_secs(7);
const ERR: Duration = Duration::from_secs(9);
const MAX: usize = 5;

#[derive(Clone, PartialEq)]
struct Toast {
    id: u64,
    kind: &'static str,
    title: String,
    description: Option<String>,
}

#[derive(Clone, Copy)]
pub struct Toasts {
    list: Signal<Vec<Toast>>,
    next: Signal<u64>,
}

impl Toasts {
    pub fn success(self, title: String) {
        self.push("success", title, None, OK);
    }

    pub fn warn(self, title: String) {
        self.push("warn", title, None, WARN);
    }

    pub fn error(self, title: String, description: String) {
        self.push("error", title, Some(description), ERR);
    }

    fn push(
        mut self,
        kind: &'static str,
        title: String,
        description: Option<String>,
        duration: Duration,
    ) {
        let id = self.next.replace(self.next.cloned() + 1);

        self.list.with_mut(|list| {
            list.push(Toast {
                id,
                kind,
                title,
                description,
            });

            if list.len() > MAX {
                list.remove(0);
            }
        });

        dioxus::core::spawn_forever(async move {
            TimeoutFuture::new(u32::try_from(duration.as_millis()).unwrap_or(u32::MAX)).await;
            self.close(id);
        });
    }

    fn close(mut self, id: u64) {
        self.list.with_mut(|list| list.retain(|toast| toast.id != id));
    }
}

#[component]
pub fn ToastProvider(children: Element) -> Element {
    let toasts = use_context_provider(|| Toasts {
        list: Signal::new(Vec::new()),
        next: Signal::new(0),
    });

    rsx! {
        {children}

        div { class: "toasts", aria_live: "polite",
            for Toast { id, kind, title, description } in toasts.list.read().clone() {
                div { key: "{id}", class: "toast toast--{kind}",
                    div { class: "toast-body",
                        span { class: "toast-title", "{title}" }

                        if let Some(description) = description {
                            span { class: "toast-description", "{description}" }
                        }
                    }

                    button {
                        class: "toast-close",
                        onclick: move |_| toasts.close(id),
                        Icon { name: "close" }
                    }
                }
            }
        }
    }
}