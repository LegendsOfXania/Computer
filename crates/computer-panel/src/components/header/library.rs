use crate::{
    components::{btn::Btn, icon::Icon, toast::Toasts},
    i18n::I18n,
    id,
    state::{app::AppState, ui::UiState},
    ws::Client,
};

use super::tree::{flatten, Row};
use computer_model::{
    page::{Page, PageKind},
    protocol::request::Request,
};
use dioxus::{prelude::*, signals::ReadableExt};

const NONE: &str = "—";

#[component]
pub fn Library() -> Element {
    let i18n = use_context::<I18n>();
    let app = use_context::<AppState>();
    let ui = use_context::<UiState>();
    let toasts = use_context::<Toasts>();
    let client = use_context::<Client>();

    let rows = flatten(app.pages.read().values());
    let opened = (ui.opened_page)();

    let (name, id, kind, priority, chapter) = opened
        .and_then(|id| {
            app.with_page(id, |p| {
                (
                    p.name.clone(),
                    p.id.to_string(),
                    p.kind.as_str().to_string(),
                    p.priority.to_string(),
                    p.chapter.clone().unwrap_or_else(|| NONE.into()),
                )
            })
        })
        .unwrap_or_else(|| {
            (
                i18n.t("header.library.opened_page.none"),
                NONE.into(),
                NONE.into(),
                NONE.into(),
                NONE.into(),
            )
        });

    let create_page = {
        let (client, toasts, i18n) = (client.clone(), toasts.clone(), i18n.clone());

        move |_| {
            let (client, toasts, i18n) = (client.clone(), toasts.clone(), i18n.clone());

            spawn(async move {
                let page = Page {
                    id: id::generate(),
                    name: "New page".into(),
                    kind: PageKind::Static,
                    priority: 0,
                    chapter: None,
                    entries: Vec::new(),
                };

                if let Err(e) = client.request(Request::CreatePage { page }).await {
                    toasts.push(3, i18n.t("toast.create_page.failed"), e);
                }
            });
        }
    };

    rsx! {
        document::Stylesheet { href: asset!("/assets/style/header/library.css") }

        div { class: "library",
            button { class: "library-trigger",
                Icon { name: "library" }
                span { {i18n.t("header.library")} }
            }

            div { class: "library-panel",
                section { class: "library-col library-list",
                    for (i, row) in rows.into_iter().enumerate() {
                        match row {
                            Row::Chapter { name, depth } => rsx! {
                                p {
                                    key: "{i}",
                                    class: "library-chapter",
                                    style: "--depth: {depth}",
                                    "{name}"
                                }
                            },
                            Row::Page { id, name, depth } => rsx! {
                                p {
                                    key: "{i}",
                                    class: if opened == Some(id) { "library-page library-opened" } else { "library-page" },
                                    style: "--depth: {depth}",
                                    "{name}"
                                }
                            },
                        }
                    }
                }

                section { class: "library-col library-details",
                    div { class: "library-head",
                        p { class: "library-name",
                            Icon { name: "book-open" }
                            "{name}"
                        }
                        p { class: "library-id", "{id}" }
                    }

                    div { class: "library-meta",
                        p {
                            span { class: "library-label",
                                Icon { name: "git-commit" }
                                {i18n.t("header.library.opened_page.kind")}
                            }
                            span { "{kind}" }
                        }

                        p {
                            span { class: "library-label",
                                Icon { name: "arrow-big-up-dash" }
                                {i18n.t("header.library.opened_page.priority")}
                            }
                            span { "{priority}" }
                        }

                        p {
                            span { class: "library-label",
                                Icon { name: "folder" }
                                {i18n.t("header.library.opened_page.chapter")}
                            }
                            span { "{chapter}" }
                        }
                    }

                    div { class: "library-footer",
                        Btn {
                            icon: "plus",
                            tooltip: Some(i18n.t("header.library.tooltip.create_page")),
                            onclick: create_page,
                        }

                        Btn {
                            icon: "trash",
                            tooltip: Some(i18n.t("header.library.tooltip.delete_page")),
                        }
                    }
                }
            }
        }
    }
}