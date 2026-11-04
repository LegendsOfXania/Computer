mod body;
mod display;
mod field;

use computer_model::{Entry, value::Value};
use dioxus::prelude::*;
use dioxus_i18n::tid;
use dioxus_primitives::toast::{ToastOptions, use_toast};

use crate::{
    components::{inspector::body::Body, utils::toast::ERR},
    state::{app::AppState, ui::UiState},
    ws::Client,
};

#[component]
pub fn Inspector() -> Element {
    let (app, mut ui, client, toast) = (
        use_context::<AppState>(),
        use_context::<UiState>(),
        use_context::<Client>(),
        use_toast(),
    );

    let mut revision = use_signal(|| 0u32);

    let fail = use_callback(move |error: String| {
        revision.set(revision() + 1);

        toast.error(
            tid!("toast-edit-entry-failed"),
            ToastOptions::new().description(error).duration(ERR),
        );
    });

    let edit = use_callback(move |(name, value): (String, Value)| {
        let Some(id) = ui.get_opened_entry() else {
            return;
        };

        let client = client.clone();

        spawn(async move {
            let change = move |entry: &mut Entry| {
                entry.fields.insert(name, value);
            };

            if let Err(error) = client.edit_entry(&app, id, change).await {
                fail.call(error);
            }
        });
    });

    let entry = use_memo(move || {
        ui.get_opened_entry()
            .and_then(|id| app.with_entry(&id, Clone::clone))
    });

    let definition =
        use_memo(move || entry().and_then(|entry| app.registry.read().get(&entry.kind).cloned()));

    let is_open = entry.read().is_some();

    rsx! {
        document::Stylesheet { href: asset!("/assets/style/inspector/mod.css") }

        aside { class: if is_open { "inspector open" } else { "inspector" },
            match (entry(), definition()) {
                (Some(entry), Some(definition)) => rsx! {
                    Body {
                        entry,
                        definition,
                        revision: revision(),
                        edit,
                        close: move |_| ui.set_opened_entry(&app, None),
                    }
                },
                (Some(_), None) => rsx! {
                    p { class: "inspector-empty", {tid!("inspector-unknown-kind")} }
                },
                _ => rsx! {},
            }
        }
    }
}
