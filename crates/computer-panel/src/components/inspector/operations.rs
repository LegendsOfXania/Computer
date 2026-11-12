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

fn add_reference(value: &mut Value, source: &EntryId, duplicate: &EntryId) -> bool {
    match value {
        Value::List(values) => {
            let (has_src, has_dup) = (
                values.iter().any(|v| matches!(v, Value::Reference(id) if id == source)),
                values.iter().any(|v| matches!(v, Value::Reference(id) if id == duplicate)),
            );
            let mut changed = false;
            if has_src && !has_dup {
                values.push(Value::Reference(duplicate.clone()));
                changed = true;
            }
            values.iter_mut().fold(changed, |acc, v| add_reference(v, source, duplicate) || acc)
        }
        Value::Struct(fields) | Value::Variant { fields, .. } => {
            fields.values_mut().fold(false, |acc, v| add_reference(v, source, duplicate) || acc)
        }
        Value::Map(pairs) => {
            pairs.iter_mut().fold(false, |acc, (k, v)| {
                add_reference(k, source, duplicate) || add_reference(v, source, duplicate) || acc
            })
        }
        _ => false,
    }
}

#[component]
pub fn Operations(entry: Entry, definition: EntryDefinition) -> Element {
    let (app, mut ui, client, toast) = (
        use_context::<AppState>(),
        use_context::<UiState>(),
        use_context::<Client>(),
        use_toast(),
    );

    let notify_err = move |key: &'static str, err: String| {
        toast.error(tid!(key), ToastOptions::new().description(err).duration(ERR));
    };

    let pick_page = {
        let (client, notify_err, entry_id) = (client.clone(), notify_err, entry.id.clone());
        use_callback(move |id: EntryId| {
            let (client, notify_err, entry_id) = (client.clone(), notify_err, entry_id.clone());
            let page_id = PageId::from_string(id.into_string());
            spawn(async move {
                if let Err(e) = client.move_entry(entry_id.clone(), page_id).await {
                    notify_err("toast-edit-entry-failed", e);
                } else {
                    ui.set_opened_entry(&app, Some(entry_id));
                }
            });
        })
    };

    let on_delete = {
        let (client, notify_err, source) = (client.clone(), notify_err, entry.clone());
        move |_| {
            let (client, notify_err, source) = (client.clone(), notify_err, source.clone());
            spawn(async move {
                let mut eval = document::eval("dioxus.send(window.confirm('Are you sure you want to delete this entry?'))");
                if matches!(eval.recv::<bool>().await, Ok(true)) {
                    match client.delete_entry(source.id.clone()).await {
                        Ok(()) => {
                            if ui.get_opened_entry().as_ref() == Some(&source.id) {
                                ui.set_opened_entry(&app, None);
                            }
                        }
                        Err(e) => notify_err("toast-delete-entry-failed", e),
                    }
                }
            });
        }
    };

    let duplicate_action = |linked: bool| {
        let (client, notify_err, source, definition) = (client.clone(), notify_err, entry.clone(), definition.clone());
        move |_| {
            let (client, notify_err, source, definition) = (client.clone(), notify_err, source.clone(), definition.clone());
            spawn(async move {
                let page_id = match app.pages.read().iter().find(|(_, p)| p.entries.contains(&source.id)).map(|(id, _)| id.clone()) {
                    Some(id) => id,
                    None => return,
                };

                let new_id = match client.create_entry(&definition, page_id).await {
                    Ok(id) => id,
                    Err(e) => { notify_err("toast-create-entry-failed", e); return; }
                };

                if let Err(e) = client.edit_entry(&app, new_id.clone(), move |d| d.fields = source.fields.clone()).await {
                    notify_err("toast-edit-entry-failed", e);
                    return;
                }

                if linked {
                    let updates: Vec<_> = app.entries.read().iter()
                        .filter(|(pid, _)| *pid != &new_id)
                        .filter_map(|(pid, p)| {
                            let mut fields = p.fields.clone();
                            let changed = fields.values_mut().fold(false, |acc, v| add_reference(v, &source.id, &new_id) || acc);
                            changed.then_some((pid.clone(), fields))
                        })
                        .collect();

                    for (pid, fields) in updates {
                        if let Err(e) = client.edit_entry(&app, pid, move |p| p.fields = fields).await {
                            notify_err("toast-edit-entry-failed", e);
                            return;
                        }
                    }
                }

                ui.set_opened_entry(&app, Some(new_id));
            });
        }
    };

    rsx! {
        div { class: "inspector-operations",
            Btn {
                icon: "trash",
                label: tid!("inspector-delete"),
                class: "inspector-operation",
                popover_target: Some("inspector-operations".into()),
                popover_action: Some("hide".into()),
                onclick: on_delete
            }

            Btn {
                icon: "section-plus",
                label: tid!("inspector-duplicate"),
                class: "inspector-operation",
                popover_target: Some("inspector-operations".into()),
                popover_action: Some("hide".into()),
                onclick: duplicate_action(false)
            }

            Btn {
                icon: "link",
                label: tid!("inspector-duplicate-link"),
                class: "inspector-operation",
                popover_target: Some("inspector-operations".into()),
                popover_action: Some("hide".into()),
                onclick: duplicate_action(true)
            }

            Btn {
                icon: "move",
                label: tid!("inspector-move"),
                class: "inspector-operation",
                popover_target: Some("inspector-operations".into()),
                popover_action: Some("hide".into()),
                onclick: move |_| {
                    ui.set_search(Some((PAGE | PAGE_PICK, pick_page)));
                }
            }
        }
    }
}