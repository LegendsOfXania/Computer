mod body;
mod field;
mod operations;

use computer_model::{Entry, Value};
use dioxus::prelude::*;
use dioxus_i18n::tid;

use crate::{
    actions::Actions,
    components::inspector::body::Body,
    state::{app::AppState, ui::UiState},
};

const OPERATIONS: &str = "inspector-operations";

#[component]
pub fn Inspector() -> Element {
    let (app, mut ui, actions) = (
        use_context::<AppState>(),
        use_context::<UiState>(),
        use_context::<Actions>(),
    );

    let mut revision = use_signal(|| 0u32);

    let edit = use_callback(move |(name, subtree): (String, Vec<Value>)| {
        let Some(id) = ui.get_opened_entry() else {
            return;
        };

        spawn(async move {
            let change = move |entry: &mut Entry| {
                entry.set(&name, subtree);
            };

            if !actions.edit_entry(id, change).await {
                revision.set(revision() + 1);
            }
        });
    });

    let entry = use_memo(move || {
        ui.get_opened_entry()
            .and_then(|id| app.with_entry(id, Entry::clone))
    });

    let definition = use_memo(move || entry().and_then(|entry| app.definition(&entry.kind)));

    let is_open = entry.read().is_some();

    rsx! {

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
