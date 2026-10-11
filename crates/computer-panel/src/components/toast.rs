use std::time::Duration;

use dioxus::prelude::*;

use crate::components::icon::Icon;

#[derive(Clone, PartialEq)]
struct Item {
    id: u64,
    level: u8,
    title: String,
    message: String,
    duration: Duration,
}

#[derive(Clone, Copy)]
pub struct Toasts {
    items: Signal<Vec<Item>>,
    next: Signal<u64>,
}

impl Toasts {
    pub fn push(&self, level: u8, title: impl Into<String>, message: impl Into<String>) {
        let secs = match level {
            0 | 1 => 4,
            2 => 6,
            _ => 8,
        };
        self.push_for(level, title, message, Duration::from_secs(secs));
    }

    pub fn push_for(
        &self,
        level: u8,
        title: impl Into<String>,
        message: impl Into<String>,
        duration: Duration,
    ) {
        let (mut items, mut next) = (self.items, self.next);
        let id = *next.peek();
        *next.write() += 1;
        items.write().push(Item {
            id,
            level,
            title: title.into(),
            message: message.into(),
            duration,
        });
    }

    fn remove(&self, id: u64) {
        let mut items = self.items;
        items.write().retain(|t| t.id != id);
    }
}

#[component]
pub fn ToastProvider(children: Element) -> Element {
    let toasts = use_context_provider(|| Toasts {
        items: Signal::new(Vec::new()),
        next: Signal::new(0),
    });

    rsx! {
        {children}
        div { class: "toasts",
            for t in toasts.items.read().iter().rev() {
                Toast {
                    key: "{t.id}",
                    id: t.id,
                    level: t.level,
                    title: t.title.clone(),
                    message: t.message.clone(),
                    duration: t.duration,
                }
            }
        }
    }
}

#[component]
fn Toast(id: u64, level: u8, title: String, message: String, duration: Duration) -> Element {
    let toasts = use_context::<Toasts>();
    let level = level.min(3);
    let icon = ["chevron-right", "check", "warning-diamond", "close"][level as usize];
    let t = duration.as_secs_f32();

    rsx! {
        div {
            class: "toast",
            style: "--t:{t}s",
            onanimationend: move |e| {
                if e.animation_name() == "toast-collapse" {
                    toasts.remove(id);
                }
            },
            div { class: "toast-s",
                div { class: "toast-b l{level}",
                    div { class: "t-h",
                        Icon { name: icon }
                        span { "{title}" }
                    }
                    div { class: "t-m", "{message}" }
                }
            }
        }
    }
}