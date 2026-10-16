use crate::{
    components::{btn::Btn, icon::Icon, toast::Toasts},
    state::{app::AppState, ui::UiState},
    ws::Client,
};

use dioxus::prelude::*;
use dioxus_i18n::tid;

const NONE: &str = "—";

type Ctx = (AppState, Client, Toasts);

#[component]
pub fn Details() -> Element {
    let app = use_context::<AppState>();
    let ui = use_context::<UiState>();
    let ctx: Ctx = (app.clone(), use_context::<Client>(), use_context::<Toasts>());

    let mut opened_page = ui.opened_page;
    let mut opened_entry = ui.opened_entry;

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

    rsx! {
        section { class: "library-col library-details",
            div { class: "library-head",
                p { class: "library-name",
                    Icon { name: "book-open" }
                    input {
                        class: "library-name-input",
                        value: "{name}",
                        disabled: none,
                        onchange: {
                            let app = app.clone();
                            let client = ctx.1.clone();
                            let toasts = ctx.2.clone();

                            move |e| {
                                let (app, client, toasts) =
                                    (app.clone(), client.clone(), toasts.clone());
                                let value = e.value();
                                let Some(id) = opened_page() else { return };

                                spawn(async move {
                                    if let Err(e) = client
                                        .edit_page(&app, id, |p| p.name = value)
                                        .await
                                    {
                                        toasts.push(3, tid!("toast-edit-page-failed"), e);
                                    }
                                });
                            }
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
                        onchange: {
                            let app = app.clone();
                            let client = ctx.1.clone();
                            let toasts = ctx.2.clone();

                            move |e| {
                                let (app, client, toasts) =
                                    (app.clone(), client.clone(), toasts.clone());
                                let Ok(value) = e.value().parse() else { return };
                                let Some(id) = opened_page() else { return };

                                spawn(async move {
                                    if let Err(e) = client
                                        .edit_page(&app, id, |p| p.priority = value)
                                        .await
                                    {
                                        toasts.push(3, tid!("toast-edit-page-failed"), e);
                                    }
                                });
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
                        onchange: {
                            let app = app.clone();
                            let client = ctx.1.clone();
                            let toasts = ctx.2.clone();

                            move |e| {
                                let (app, client, toasts) =
                                    (app.clone(), client.clone(), toasts.clone());
                                let value = e.value();
                                let Some(id) = opened_page() else { return };

                                spawn(async move {
                                    if let Err(e) = client
                                        .edit_page(&app, id, |p| {
                                            p.chapter = (!value.trim().is_empty()).then_some(value);
                                        })
                                        .await
                                    {
                                        toasts.push(3, tid!("toast-edit-page-failed"), e);
                                    }
                                });
                            }
                        },
                    }
                }
            }

            div { class: "library-footer",
                Btn {
                    icon: "plus",
                    tooltip: Some(tid!("header-library-tooltip-create-page")),
                    onclick: {
                        let c = ctx.clone();
                        move |_| {
                            let (_, client, toasts) = c.clone();

                            spawn(async move {
                                if let Err(e) = client
                                    .create_page(tid!("library-new-page-name"))
                                    .await
                                {
                                    toasts.push(3, tid!("toast-create-page-failed"), e);
                                }
                            });
                        }
                    },
                }

                Btn {
                    icon: "trash",
                    tooltip: Some(tid!("header-library-tooltip-delete-page")),
                    onclick: {
                        let c = ctx.clone();
                        move |_| {
                            let Some(id) = opened_page() else { return };
                            let (app, client, toasts) = c.clone();

                            let mut pages: Vec<_> = app
                                .pages
                                .read()
                                .values()
                                .map(|p| (p.priority, p.id))
                                .collect();
                            pages.sort_unstable();

                            let ids: Vec<u64> =
                                pages.into_iter().map(|(_, id)| id).collect();

                            let next = ids
                                .iter()
                                .position(|&i| i == id)
                                .and_then(|pos| {
                                    ids.get(pos + 1).or_else(|| ids.get(pos.checked_sub(1)?))
                                })
                                .copied();

                            spawn(async move {
                                match client.delete_page(id).await {
                                    Ok(()) => {
                                        if opened_page() == Some(id) {
                                            opened_page.set(next);
                                            opened_entry.set(None);
                                        }
                                    }
                                    Err(e) => {
                                        toasts.push(3, tid!("toast-delete-page-failed"), e);
                                    }
                                }
                            });
                        }
                    },
                }
            }
        }
    }
}