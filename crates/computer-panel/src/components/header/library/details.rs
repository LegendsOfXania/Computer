use crate::{
    components::utils::{btn::Btn, icon::Icon, toast::ERR},
    state::{app::AppState, ui::UiState},
    ws::Client,
};
use computer_model::{id::Identifier, page::Page};
use dioxus::prelude::*;
use dioxus_i18n::tid;
use dioxus_primitives::toast::{ToastOptions, use_toast};

const NONE: &str = "—";

#[component]
pub fn Details() -> Element {
    let (app, client, ui, toast) = (
        use_context::<AppState>(),
        use_context::<Client>(),
        use_context::<UiState>(),
        use_toast(),
    );

    let mut opened_page = ui.opened_page;

    let fail = use_callback(move |(title, error): (String, String)| {
        toast.error(title, ToastOptions::new().description(error).duration(ERR));
    });

    let edit = use_callback({
        let (app, client) = (app.clone(), client.clone());

        move |f: Box<dyn FnOnce(&mut Page)>| {
            let Some(id) = opened_page() else {
                return;
            };

            let (app, client) = (app.clone(), client.clone());

            spawn(async move {
                if let Err(error) = client.edit_page(&app, id, f).await {
                    fail.call((tid!("toast-edit-page-failed"), error));
                }
            });
        }
    });

    let (name, id, kind, priority, chapter) = opened_page()
        .and_then(|id| {
            app.with_page(&id, |page| {
                (
                    page.name.clone(),
                    page.id.clone().into_string(),
                    page.kind.as_str().to_string(),
                    page.priority.to_string(),
                    page.chapter.clone().unwrap_or_default(),
                )
            })
        })
        .unwrap_or_else(|| {
            (
                tid!("header-library-opened-page-none"),
                NONE.to_string(),
                NONE.to_string(),
                String::new(),
                String::new(),
            )
        });

    let none = opened_page().is_none();
    let placeholder = if none { NONE } else { "" };

    let create = {
        let client = client.clone();

        move |_| {
            let client = client.clone();

            spawn(async move {
                if let Err(error) = client.create_page(tid!("library-new-page-name")).await {
                    fail.call((tid!("toast-create-page-failed"), error));
                }
            });
        }
    };

    let delete = {
        let app = app.clone();

        move |_| {
            let Some(id) = opened_page() else {
                return;
            };

            let client = client.clone();

            let mut pages: Vec<_> = app
                .pages
                .read()
                .values()
                .map(|page| (page.priority, page.id.clone()))
                .collect();

            pages.sort_by_key(|(priority, _)| *priority);

            let next = pages
                .iter()
                .position(|(_, page_id)| page_id == &id)
                .and_then(|index| {
                    pages.get(index + 1).or_else(|| {
                        index
                            .checked_sub(1)
                            .and_then(|previous| pages.get(previous))
                    })
                })
                .map(|(_, page_id)| page_id.clone());

            spawn(async move {
                match client.delete_page(id.clone()).await {
                    Ok(()) if opened_page().as_ref() == Some(&id) => {
                        opened_page.set(next);
                    }
                    Ok(()) => {}
                    Err(error) => {
                        fail.call((tid!("toast-delete-page-failed"), error));
                    }
                }
            });
        }
    };

    rsx! {
        section { class: "library-col library-details",
            div { class: "library-head",
                p { class: "library-name",
                    Icon { name: "book-open" }
                    input {
                        class: "library-name-input",
                        value: "{name}",
                        disabled: none,
                        onchange: move |event| {
                            let value = event.value();
                            edit.call(Box::new(move |page| page.name = value));
                        },
                    }
                }
                p { class: "library-id", "{id}" }
            }

            div { class: "library-meta",
                p {
                    span { class: "library-label",
                        Icon { name: "git-commit" }
                        {tid!("header-library-opened-page-kind")}
                    }
                    span { "{kind}" }
                }
                p {
                    span { class: "library-label",
                        Icon { name: "arrow-big-up-dash" }
                        {tid!("header-library-opened-page-priority")}
                    }
                    input {
                        r#type: "number",
                        value: "{priority}",
                        placeholder,
                        disabled: none,
                        onchange: move |event| {
                            if let Ok(value) = event.value().parse::<i32>() {
                                edit.call(Box::new(move |page| page.priority = value));
                            }
                        },
                    }
                }
                p {
                    span { class: "library-label",
                        Icon { name: "folder" }
                        {tid!("header-library-opened-page-chapter")}
                    }
                    input {
                        value: "{chapter}",
                        placeholder,
                        disabled: none,
                        onchange: move |event| {
                            let value = event.value();
                            edit.call(Box::new(move |page| {
                                page.chapter = (!value.trim().is_empty()).then_some(value);
                            }));
                        },
                    }
                }
            }

            div { class: "library-footer",
                Btn {
                    icon: "plus",
                    tooltip: tid!("header-library-tooltip-create-page"),
                    onclick: create,
                }
                Btn {
                    icon: "trash",
                    tooltip: tid!("header-library-tooltip-delete-page"),
                    onclick: delete,
                }
            }
        }
    }
}
