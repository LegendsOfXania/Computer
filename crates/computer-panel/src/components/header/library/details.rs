use crate::{
    components::{btn::Btn, icon::Icon, toast::Toasts},
    i18n::I18n,
    state::{app::AppState, ui::UiState},
    ws::Client,
};

use computer_model::page::Page;
use dioxus::prelude::*;

const NONE: &str = "—";

struct Info {
    name: String,
    id: String,
    kind: String,
    priority: String,
    chapter: String,
}

impl Info {
    fn of(page: &Page) -> Self {
        Self {
            name: page.name.clone(),
            id: page.id.to_string(),
            kind: page.kind.as_str().to_string(),
            priority: page.priority.to_string(),
            chapter: page.chapter.clone().unwrap_or_default(),
        }
    }

    fn none(i18n: &I18n) -> Self {
        Self {
            name: i18n.t("header.library.opened_page.none"),
            id: NONE.into(),
            kind: NONE.into(),
            priority: NONE.into(),
            chapter: String::new(),
        }
    }
}

#[component]
pub fn Details() -> Element {
    let i18n = use_context::<I18n>();
    let app = use_context::<AppState>();
    let ui = use_context::<UiState>();
    let toasts = use_context::<Toasts>();
    let client = use_context::<Client>();

    let mut opened_page = ui.opened_page;
    let mut opened_entry = ui.opened_entry;

    let page_id = opened_page();

    let info = page_id
        .and_then(|id| app.with_page(id, Info::of))
        .unwrap_or_else(|| Info::none(&i18n));

    let page_ids: Vec<u64> = {
        let pages = app.pages.read();
        let mut pages: Vec<_> = pages
            .values()
            .map(|page| (page.priority, page.id))
            .collect();

        pages.sort_unstable();

        pages.into_iter().map(|(_, id)| id).collect()
    };

    rsx! {
        section { class: "library-col library-details",
            div { class: "library-head",
                p { class: "library-name",
                    Icon { name: "book-open" }
                    input {
                        class: "library-name-input",
                        value: "{info.name}",
                        disabled: page_id.is_none(),
                        onchange: {
                            let (app, client, toasts, i18n) =
                                (app.clone(), client.clone(), toasts.clone(), i18n.clone());

                            move |event| {
                                let Some(id) = page_id else { return };
                                let Some(mut page) = app.with_page(id, Clone::clone) else {
                                    return;
                                };

                                page.name = event.value();

                                let (client, toasts, i18n) =
                                    (client.clone(), toasts.clone(), i18n.clone());

                                spawn(async move {
                                    if let Err(e) = client.edit_page(page).await {
                                        toasts.push(3, i18n.t("toast.edit_page.failed"), e);
                                    }
                                });
                            }
                        },
                    }
                }
                p { class: "library-id", "{info.id}" }
            }

            div { class: "library-meta",
                p {
                    span { class: "library-label",
                        Icon { name: "git-commit" }
                        {i18n.t("header.library.opened_page.kind")}
                    }
                    span { "{info.kind}" }
                }

                p {
                    span { class: "library-label",
                        Icon { name: "arrow-big-up-dash" }
                        {i18n.t("header.library.opened_page.priority")}
                    }
                    input {
                        r#type: "number",
                        value: "{info.priority}",
                        disabled: page_id.is_none(),
                        onchange: {
                            let (app, client, toasts, i18n) =
                                (app.clone(), client.clone(), toasts.clone(), i18n.clone());

                            move |event| {
                                let Some(id) = page_id else { return };
                                let Some(mut page) = app.with_page(id, Clone::clone) else {
                                    return;
                                };
                                let Ok(priority) = event.value().parse() else {
                                    return;
                                };

                                page.priority = priority;

                                let (client, toasts, i18n) =
                                    (client.clone(), toasts.clone(), i18n.clone());

                                spawn(async move {
                                    if let Err(e) = client.edit_page(page).await {
                                        toasts.push(3, i18n.t("toast.edit_page.failed"), e);
                                    }
                                });
                            }
                        },
                    }
                }

                p {
                    span { class: "library-label",
                        Icon { name: "folder" }
                        {i18n.t("header.library.opened_page.chapter")}
                    }
                    input {
                        value: "{info.chapter}",
                        disabled: page_id.is_none(),
                        onchange: {
                            let (app, client, toasts, i18n) =
                                (app.clone(), client.clone(), toasts.clone(), i18n.clone());

                            move |event| {
                                let Some(id) = page_id else { return };
                                let Some(mut page) = app.with_page(id, Clone::clone) else {
                                    return;
                                };

                                let value = event.value();
                                page.chapter = if value.trim().is_empty() {
                                    None
                                } else {
                                    Some(value)
                                };

                                let (client, toasts, i18n) =
                                    (client.clone(), toasts.clone(), i18n.clone());

                                spawn(async move {
                                    if let Err(e) = client.edit_page(page).await {
                                        toasts.push(3, i18n.t("toast.edit_page.failed"), e);
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
                    tooltip: Some(i18n.t("header.library.tooltip.create_page")),
                    onclick: {
                        let (client, toasts, i18n) =
                            (client.clone(), toasts.clone(), i18n.clone());

                        move |_| {
                            let (client, toasts, i18n) =
                                (client.clone(), toasts.clone(), i18n.clone());

                            spawn(async move {
                                if let Err(e) = client.create_page("New page".into()).await {
                                    toasts.push(3, i18n.t("toast.create_page.failed"), e);
                                }
                            });
                        }
                    },
                }

                Btn {
                    icon: "trash",
                    tooltip: Some(i18n.t("header.library.tooltip.delete_page")),
                    onclick: {
                        let (client, toasts, i18n) =
                            (client.clone(), toasts.clone(), i18n.clone());
                        let page_ids = page_ids.clone();

                        move |_| {
                            let Some(page_id) = opened_page() else {
                                return;
                            };

                            let next = page_ids
                                .iter()
                                .position(|id| *id == page_id)
                                .and_then(|pos| {
                                    page_ids
                                        .get(pos + 1)
                                        .or_else(|| {
                                            pos.checked_sub(1)
                                                .and_then(|prev| page_ids.get(prev))
                                        })
                                })
                                .copied();

                            let (client, toasts, i18n) =
                                (client.clone(), toasts.clone(), i18n.clone());

                            spawn(async move {
                                match client.delete_page(page_id).await {
                                    Ok(()) => {
                                        if opened_page() == Some(page_id) {
                                            opened_page.set(next);
                                            opened_entry.set(None);
                                        }
                                    }
                                    Err(e) => {
                                        toasts.push(
                                            3,
                                            i18n.t("toast.delete_page.failed"),
                                            e,
                                        );
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