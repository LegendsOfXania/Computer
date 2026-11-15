use computer_model::{Entry, EntryDefinition};
use dioxus::prelude::*;
use dioxus_i18n::tid;

use crate::{
    actions::Actions,
    components::{inspector::OPERATIONS, utils::btn::Btn},
    search::PAGE,
    state::ui::{Picked, SearchRequest, UiState},
};

fn operation(icon: &'static str, label: String, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        Btn {
            icon,
            label,
            class: "inspector-operation",
            popover_target: Some(OPERATIONS.to_string()),
            popover_action: Some("hide".to_string()),
            onclick,
        }
    }
}

#[component]
pub fn Operations(entry: Entry, definition: EntryDefinition) -> Element {
    let (mut ui, actions) = (use_context::<UiState>(), use_context::<Actions>());
    let id = entry.id;

    let pick = use_callback(move |picked: Picked| {
        if let Picked::Page(page) = picked {
            spawn(async move { actions.move_entry(id, page).await });
        }
    });

    let duplicate = move |linked: bool| {
        let entry = entry.clone();

        EventHandler::new(move |_| {
            let entry = entry.clone();

            spawn(async move { actions.duplicate_entry(entry, linked).await });
        })
    };

    rsx! {
        div { class: "inspector-operations",
            {
                operation(
                    "trash",
                    tid!("inspector-delete"),
                    EventHandler::new(move |_| {
                        spawn(async move { actions.delete_entry(id).await });
                    }),
                )
            }
            {operation("section-plus", tid!("inspector-duplicate"), duplicate(false))}
            {operation("link", tid!("inspector-duplicate-link"), duplicate(true))}
            {
                operation(
                    "move",
                    tid!("inspector-move"),
                    EventHandler::new(move |_| {
                        ui.set_search(
                            Some(SearchRequest {
                                scope: PAGE,
                                accepts: Some(definition.clone()),
                                pick,
                            }),
                        );
                    }),
                )
            }
        }
    }
}
