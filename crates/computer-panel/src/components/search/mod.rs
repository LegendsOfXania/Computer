mod category;
mod hit;

use computer_model::{EntryDefinition, page::PageKind};
use dioxus::prelude::*;
use dioxus_i18n::tid;

use category::Category;

use crate::{
    actions::Actions,
    components::utils::icon::Icon,
    search::{self, Target, build_index, build_sections},
    state::{
        app::AppState,
        ui::{Picked, SearchRequest, UiState},
    },
};

#[component]
pub fn Search(request: SearchRequest) -> Element {
    let (app, mut ui, actions) = (
        use_context::<AppState>(),
        use_context::<UiState>(),
        use_context::<Actions>(),
    );

    let SearchRequest {
        scope,
        accepts,
        pick,
    } = request;

    let index = use_memo(move || {
        build_index(
            &app.library.read(),
            &app.registry.read(),
            &tid!("search-new-entry"),
        )
    });

    let mut query = use_signal(String::new);
    let mut pending = use_signal(|| None::<EntryDefinition>);
    let mut collapsed = use_signal(|| [false; 3]);

    let filter = accepts.clone();

    let sections = use_memo(move || {
        let definition = pending.read().clone().or_else(|| filter.clone());

        build_sections(
            &app.library.read(),
            &index.read(),
            &query.read(),
            scope,
            definition.as_ref(),
            &tid!("header-library-tooltip-create-page"),
        )
    });

    let wanted = accepts.clone();

    let activate = use_callback(move |target: Target| {
        let wanted = wanted.clone();

        spawn(async move {
            let mut definition = pending();

            let page = match target {
                Target::Entry(id) => {
                    pick.call(Picked::Entry(id));
                    return ui.set_search(None);
                }

                Target::Page(id) if definition.is_none() => {
                    pick.call(Picked::Page(id));
                    return ui.set_search(None);
                }

                Target::Page(id) => id,

                Target::NewEntry(kind) => {
                    let Some(chosen) = app.definition(&kind) else {
                        return;
                    };

                    let opened = ui.get_opened_page().filter(|id| {
                        app.with_page(*id, |page| page.kind.accepts(&chosen)) == Some(true)
                    });

                    let Some(id) = opened else {
                        pending.set(Some(chosen));
                        return query.set(String::new());
                    };

                    definition = Some(chosen);
                    id
                }

                Target::NewPage => {
                    let kind = match definition.as_ref().or(wanted.as_ref()) {
                        Some(chosen) => PageKind::ALL.into_iter().find(|kind| kind.accepts(chosen)),
                        None => Some(PageKind::default()),
                    };

                    let Some(id) = (match kind {
                        Some(kind) => actions.create_page(kind).await,
                        None => None,
                    }) else {
                        return;
                    };

                    if definition.is_none() {
                        pick.call(Picked::Page(id));
                        return ui.set_search(None);
                    }

                    id
                }
            };

            let Some(definition) = definition else {
                return;
            };

            if let Some(id) = actions.create_entry(&definition, page).await {
                pick.call(Picked::Entry(id));
                ui.set_search(None);
            }
        });
    });

    let fold = use_callback(move |category: usize| {
        collapsed.with_mut(|values| {
            if let Some(value) = values.get_mut(category) {
                *value = !*value;
            }
        });
    });

    let select = use_callback(move |(category, item): (usize, usize)| {
        let target = sections
            .read()
            .get(category)
            .and_then(|items| items.get(item))
            .map(|item| item.target.clone());

        if let Some(target) = target {
            activate.call(target);
        }
    });

    let titles = [
        tid!("search-existing-entries"),
        tid!("search-new-entries"),
        tid!("search-pages"),
    ];

    let (kind_title, extension_title, chapter_title) = (
        tid!("search-kind"),
        tid!("search-extension"),
        tid!("search-chapter"),
    );

    let placeholder = match pending.read().as_ref() {
        Some(definition) => {
            tid!("search-pick-page", kind: definition.kind.clone())
        }
        None => tid!("header-search-placeholder"),
    };

    let text = query();
    let parts = search::highlight(&text);
    let sections = sections();
    let collapsed = collapsed();

    let no_results = sections.iter().all(Vec::is_empty);

    rsx! {

        div {
            class: "search-overlay",
            onclick: move |_| ui.set_search(None),

            div {
                class: "search-box",
                onclick: move |event| event.stop_propagation(),

                div { class: "search-head",
                    label { class: "search-bar",
                        Icon { name: "search" }

                        div { class: "search-input-scroll",
                            div { class: "search-input-track",
                                span { class: "search-query-render",
                                    if text.is_empty() {
                                        span {
                                            class: "search-placeholder",
                                            "{placeholder}"
                                        }
                                    }

                                    for (index, (part, operator)) in parts.into_iter().enumerate() {
                                        span {
                                            key: "{index}",
                                            class: if operator {
                                                "search-query-operator"
                                            } else {
                                                ""
                                            },
                                            "{part}"
                                        }
                                    }
                                }

                                input {
                                    r#type: "text",
                                    class: "search-input",
                                    aria_label: "{placeholder}",
                                    autocomplete: "off",
                                    spellcheck: "false",
                                    value: "{text}",

                                    onmounted: move |event| async move {
                                        let _ = event.set_focus(true).await;
                                    },

                                    oninput: move |event| query.set(event.value()),
                                }
                            }
                        }
                    }

                    if pending.read().is_none() && accepts.is_none() {
                        div { class: "search-help",
                            span { b { "!entry" } " {titles[0]}" }
                            span { b { "!ne" } " {titles[1]}" }
                            span { b { "!page" } " {titles[2]}" }
                            span { b { "!id" } " ID" }
                            span { b { "!kind:" } " {kind_title}" }
                            span { b { "!ext:" } " {extension_title}" }
                            span { b { "!chapter:" } " {chapter_title}" }
                        }
                    }
                }

                div { class: "search-list",
                    if no_results {
                        div {
                            class: "search-empty",
                            {tid!("search-empty")}
                        }
                    }

                    for (category, items) in sections.into_iter().enumerate() {
                        if !items.is_empty() {
                            Category {
                                key: "{category}",
                                index: category,
                                title: titles.get(category).cloned().unwrap_or_default(),
                                items,
                                collapsed: collapsed.get(category).copied().unwrap_or(false),
                                ontoggle: fold,
                                onselect: select,
                            }
                        }
                    }
                }
            }
        }
    }
}
