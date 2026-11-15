use computer_model::page::{Page, PageKind};
use dioxus::prelude::*;
use dioxus_i18n::tid;

use crate::{
    actions::Actions,
    components::utils::{btn::Btn, icon::Icon},
    state::{app::AppState, ui::UiState},
};

const NONE: &str = "—";

#[component]
pub fn Details() -> Element {
    let (app, ui, actions) = (
        use_context::<AppState>(),
        use_context::<UiState>(),
        use_context::<Actions>(),
    );

    let page = use_memo(move || {
        ui.get_opened_page()
            .and_then(|id| app.with_page(id, Page::clone))
    });

    let edit = move |change: Box<dyn FnOnce(&mut Page)>| async move {
        if let Some(id) = ui.get_opened_page() {
            actions.edit_page(id, change).await;
        }
    };

    let none = page.read().is_none();

    let (name, id, kind, priority, chapter) = match page() {
        Some(page) => (
            page.name,
            page.id.to_string(),
            page.kind.as_str().to_string(),
            page.priority.to_string(),
            page.chapter.unwrap_or_default(),
        ),
        None => (
            tid!("header-library-opened-page-none"),
            NONE.to_string(),
            NONE.to_string(),
            String::new(),
            String::new(),
        ),
    };

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
                        onchange: move |event| {
                            let value = event.value();
                            edit(Box::new(move |page| page.name = value))
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
                        onchange: move |event| async move {
                            if let Ok(value) = event.value().parse::<i32>() {
                                edit(Box::new(move |page| page.priority = value)).await;
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
                            edit(
                                Box::new(move |page| {
                                    page.chapter = (!value.trim().is_empty()).then_some(value);
                                }),
                            )
                        },
                    }
                }
            }

            div { class: "library-footer",
                Btn {
                    icon: "plus",
                    tooltip: tid!("header-library-tooltip-create-page"),
                    onclick: move |_| async move {
                        actions.create_page(PageKind::default()).await;
                    },
                }
                Btn {
                    icon: "trash",
                    tooltip: tid!("header-library-tooltip-delete-page"),
                    onclick: move |_| async move {
                        if let Some(id) = ui.get_opened_page() {
                            actions.delete_page(id).await;
                        }
                    },
                }
            }
        }
    }
}
