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

#[derive(Clone, Copy, Default)]
pub struct Toasts {
    list: Signal<Vec<Toast>>,
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

    fn push(mut self, kind: &'static str, title: String, description: Option<String>, duration: Duration) {
        let id = {
            let mut list = self.list.write();
            let id = list.last().map_or(0, |toast| toast.id + 1);

            list.push(Toast {
                id,
                kind,
                title,
                description,
            });

            if list.len() > MAX {
                list.remove(0);
            }

            id
        };

        spawn(async move {
            TimeoutFuture::new(duration.as_millis().min(u32::MAX as u128) as u32).await;
            self.close(id);
        });
    }

    fn close(mut self, id: u64) {
        self.list.write().retain(|toast| toast.id != id);
    }
}

#[component]
pub fn ToastProvider(children: Element) -> Element {
    let toasts = use_context::<Toasts>();

    rsx! {
        {children}

        div {
            class: "toasts",
            aria_live: "polite",

            for toast in toasts.list.read().iter() {
                div {
                    key: "{toast.id}",
                    class: "toast toast--{toast.kind}",

                    div {
                        class: "toast-body",
                        span {
                            class: "toast-title",
                            "{toast.title}"
                        }

                        if let Some(description) = &toast.description {
                            span {
                                class: "toast-description",
                                "{description}"
                            }
                        }
                    }

                    button {
                        class: "toast-close",
                        onclick: {
                            let id = toast.id;
                            move |_| toasts.close(id)
                        },
                        Icon { name: "close" }
                    }
                }
            }
        }
    }
}