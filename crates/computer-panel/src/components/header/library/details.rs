use crate::{
    components::{btn::Btn, icon::Icon, toast::ERR},
    state::{app::AppState, ui::UiState},
    ws::Client,
};
use computer_model::page::Page;

use dioxus::prelude::*;
use dioxus_i18n::tid;
use dioxus_primitives::toast::{ToastOptions, use_toast};

const NONE: &str = "—";

#[component]
pub fn Details() -> Element {
    let app = use_context::<AppState>();
    let client = use_context::<Client>();
    let ui = use_context::<UiState>();
    let toast = use_toast();

    let mut opened_page = ui.page;

    let fail = use_callback(move |(title, e): (String, String)| {
        toast.error(title, ToastOptions::new().description(e).duration(ERR));
    });

    let edit = use_callback({
        let (app, client) = (app.clone(), client.clone());
        move |f: Box<dyn FnOnce(&mut Page)>| {
            let Some(id) = opened_page() else { return };
            let (app, client) = (app.clone(), client.clone());
            spawn(async move {
                if let Err(e) = client.edit_page(&app, id, f).await {
                    fail.call((tid!("toast-edit-page-failed"), e));
                }
            });
        }
    });

    let (name, id, kind, priority, chapter) = opened_page()
        .and_then(|id| {
            app.with_page(id, |p| {
                (
                    p.name.clone(),
                    p.id.to_string(),
                    p.kind.as_str().to_string(),
                    p.priority.to_string(),
                    p.chapter.clone().unwrap_or_default(),
                )
            })
        })
        .unwrap_or_else(|| {
            (
                tid!("header-library-opened-page-none"),
                NONE.into(),
                NONE.into(),
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
                if let Err(e) = client.create_page(tid!("library-new-page-name")).await {
                    fail.call((tid!("toast-create-page-failed"), e));
                }
            });
        }
    };

    let delete = {
        let app = app.clone();
        move |_| {
            let Some(id) = opened_page() else { return };
            let client = client.clone();

            let mut pages: Vec<_> = app
                .pages
                .read()
                .values()
                .map(|p| (p.priority, p.id))
                .collect();
            pages.sort_unstable();

            let next = pages
                .iter()
                .position(|p| p.1 == id)
                .and_then(|i| pages.get(i + 1).or_else(|| pages.get(i.checked_sub(1)?)))
                .map(|p| p.1);

            spawn(async move {
                match client.delete_page(id).await {
                    Ok(()) if opened_page() == Some(id) => opened_page.set(next),
                    Ok(()) => {}
                    Err(e) => fail.call((tid!("toast-delete-page-failed"), e)),
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
                        onchange: move |e| {
                            let v = e.value();
                            edit.call(Box::new(move |p| p.name = v));
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
                        onchange: move |e| {
                            let Ok(v) = e.value().parse::<i32>() else { return };
                            edit.call(Box::new(move |p| p.priority = v));
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
                        onchange: move |e| {
                            let v = e.value();
                            edit.call(Box::new(move |p| {
                                p.chapter = (!v.trim().is_empty()).then_some(v);
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
