
use computer_model::{
    Entry, EntryDefinition,
    id::{EntryId, Identifier, PageId},
    value::Value,
};
use dioxus::prelude::*;
use dioxus_i18n::tid;
use dioxus_primitives::toast::{ToastOptions, use_toast};

use crate::{
    components::utils::{btn::Btn, toast::ERR},
    search::{PAGE, PAGE_PICK},
    state::{app::AppState, ui::UiState},
    ws::Client,
};

#[derive(Clone, Copy)]
enum Operation {
    Delete,
    Duplicate,
    DuplicateLink,
}

fn add_reference(
    value: &mut Value,
    source: &EntryId,
    duplicate: &EntryId,
) -> bool {
    match value {
        Value::List(values) => {
            let contains_source = values.iter().any(
                |value| matches!(value, Value::Reference(id) if id == source),
            );

            let contains_duplicate = values.iter().any(
                |value| matches!(value, Value::Reference(id) if id == duplicate),
            );

            let mut changed = false;

            if contains_source && !contains_duplicate {
                values.push(Value::Reference(duplicate.clone()));
                changed = true;
            }

            for value in values {
                changed |= add_reference(value, source, duplicate);
            }

            changed
        }

        Value::Struct(fields) | Value::Variant { fields, .. } => {
            fields.values_mut().fold(false, |changed, value| {
                add_reference(value, source, duplicate) || changed
            })
        }

        Value::Map(pairs) => {
            pairs.iter_mut().fold(false, |changed, (key, value)| {
                add_reference(key, source, duplicate)
                    | add_reference(value, source, duplicate)
                    | changed
            })
        }

        _ => false,
    }
}

#[component]
pub fn Operations(entry: Entry, definition: EntryDefinition) -> Element {
    let app = use_context::<AppState>();
    let mut ui = use_context::<UiState>();
    let client = use_context::<Client>();
    let toast = use_toast();

    let move_client = client.clone();
    let move_toast = toast.clone();
    let move_entry_id = entry.id.clone();

    let pick_page = use_callback(move |id: EntryId| {
        let client = move_client.clone();
        let toast = move_toast.clone();
        let entry_id = move_entry_id.clone();
        let page_id = PageId::from_string(id.into_string());
        let mut ui = ui;

        spawn(async move {
            match client.move_entry(entry_id.clone(), page_id).await {
                Ok(()) => ui.set_opened_entry(&app, Some(entry_id)),
                Err(error) => {
                    toast.error(
                        tid!("toast-edit-entry-failed"),
                        ToastOptions::new()
                            .description(error)
                            .duration(ERR),
                    );
                }
            }
        });
    });

    let execute = use_callback(move |operation: Operation| {
        let client = client.clone();
        let toast = toast.clone();
        let source = entry.clone();
        let definition = definition.clone();
        let mut ui = ui;

        spawn(async move {
            match operation {
                Operation::Delete => {
                    let mut eval = document::eval(
                        "dioxus.send(window.confirm('Are you sure you want to delete this entry?'))",
                    );

                    if !matches!(eval.recv::<bool>().await, Ok(true)) {
                        return;
                    }

                    match client.delete_entry(source.id.clone()).await {
                        Ok(()) => {
                            if ui.get_opened_entry().as_ref() == Some(&source.id) {
                                ui.set_opened_entry(&app, None);
                            }
                        }

                        Err(error) => {
                            toast.error(
                                tid!("toast-delete-entry-failed"),
                                ToastOptions::new()
                                    .description(error)
                                    .duration(ERR),
                            );
                        }
                    }
                }

                Operation::Duplicate | Operation::DuplicateLink => {
                    let linked = matches!(operation, Operation::DuplicateLink);

                    let page_id = app.pages.read()
                        .iter()
                        .find(|(_, page)| page.entries.contains(&source.id))
                        .map(|(id, _)| id.clone());

                    let Some(page_id) = page_id else {
                        return;
                    };

                    let id = match client.create_entry(&definition, page_id).await {
                        Ok(id) => id,
                        Err(error) => {
                            toast.error(
                                tid!("toast-create-entry-failed"),
                                ToastOptions::new()
                                    .description(error)
                                    .duration(ERR),
                            );
                            return;
                        }
                    };

                    let fields = source.fields.clone();

                    if let Err(error) = client.edit_entry(
                        &app,
                        id.clone(),
                        move |duplicate| duplicate.fields = fields,
                    ).await {
                        toast.error(
                            tid!("toast-edit-entry-failed"),
                            ToastOptions::new()
                                .description(error)
                                .duration(ERR),
                        );
                        return;
                    }

                    if linked {
                        let updates = {
                            let entries = app.entries.read();

                            entries.iter()
                                .filter_map(|(parent_id, parent)| {
                                    if parent_id == &id {
                                        return None;
                                    }

                                    let mut fields = parent.fields.clone();

                                    let changed = fields.values_mut().fold(
                                        false,
                                        |changed, value| {
                                            add_reference(value, &source.id, &id)
                                                || changed
                                        },
                                    );

                                    changed.then_some((parent_id.clone(), fields))
                                })
                                .collect::<Vec<_>>()
                        };

                        for (parent_id, fields) in updates {
                            if let Err(error) = client.edit_entry(
                                &app,
                                parent_id,
                                move |parent| parent.fields = fields,
                            ).await {
                                toast.error(
                                    tid!("toast-edit-entry-failed"),
                                    ToastOptions::new()
                                        .description(error)
                                        .duration(ERR),
                                );
                                return;
                            }
                        }
                    }

                    ui.set_opened_entry(&app, Some(id));
                }
            }
        });
    });

    rsx! {
        div { class: "inspector-operations",
            Btn {
                icon: "trash",
                label: tid!("inspector-delete"),
                class: "inspector-operation",
                popover_target: Some("inspector-operations".to_string()),
                popover_action: Some("hide".to_string()),
                onclick: move |_| execute.call(Operation::Delete),
            }

            Btn {
                icon: "section-plus",
                label: tid!("inspector-duplicate"),
                class: "inspector-operation",
                popover_target: Some("inspector-operations".to_string()),
                popover_action: Some("hide".to_string()),
                onclick: move |_| execute.call(Operation::Duplicate),
            }

            Btn {
                icon: "link",
                label: tid!("inspector-duplicate-link"),
                class: "inspector-operation",
                popover_target: Some("inspector-operations".to_string()),
                popover_action: Some("hide".to_string()),
                onclick: move |_| execute.call(Operation::DuplicateLink),
            }

            Btn {
                icon: "move",
                label: tid!("inspector-move"),
                class: "inspector-operation",
                popover_target: Some("inspector-operations".to_string()),
                popover_action: Some("hide".to_string()),
                onclick: move |_| {
                    ui.set_search(Some((PAGE | PAGE_PICK, pick_page)));
                },
            }
        }
    }
}
