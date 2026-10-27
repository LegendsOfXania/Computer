mod graph;

use computer_model::id::{EntryId, Identifier, PageId};
use dioxus::prelude::*;
use dioxus_flow::prelude::*;
use std::collections::HashSet;

use crate::{components::inspector::window::Window, state::ui::UiState};
use graph::{NodeData, build};

#[component]
pub fn Editor() -> Element {
    let ui = use_context::<UiState>();

    let mut nodes = use_signal(Vec::<Node<NodeData>>::new);
    let mut edges = use_signal(Vec::<Edge>::new);
    let mut shown = use_signal(|| None::<PageId>);
    let handle = use_flow_handle::<NodeData>();

    let opened = use_memo(move || {
        ui.windows
            .read()
            .iter()
            .map(|window| window.id.clone())
            .collect::<Vec<EntryId>>()
    });

    use_effect(move || {
        let page_id = ui.page.read().clone();
        let graph = build();

        let selected: HashSet<String> = nodes
            .peek()
            .iter()
            .filter(|node| node.selected)
            .map(|node| node.id.clone())
            .collect();

        let next = graph
            .nodes
            .into_iter()
            .map(|mut node| {
                node.selected = selected.contains(&node.id);
                node
            })
            .collect();

        nodes.set(next);
        edges.set(graph.edges);

        if shown.peek().as_ref() != page_id.as_ref() {
            shown.set(page_id);
            handle.fit_view(250);
        }
    });

    rsx! {
        document::Stylesheet { href: asset!("/assets/style/editor/mod.css") }
        document::Stylesheet { href: asset!("/assets/style/editor/node.css") }
        document::Stylesheet { href: asset!("/assets/style/editor/edge.css") }

        div { style: "width: 100%; height: 100%;",
            Flow {
                nodes,
                edges,
                handle,
                fit_view: true,
                nodes_draggable: false,

                on_node_click: move |id: String| {
                    let key = nodes
                        .read()
                        .iter()
                        .find(|node| node.id == id && !node.data.foreign)
                        .map(|node| node.data.key.clone());

                    if let Some(key) = key {
                        ui.open(&key);
                    }
                },

                node_view: move |ctx: NodeViewCtx<NodeData>| {
                    let data = &ctx.node.data;
                    let open = opened.read().contains(&data.key);

                    rsx! {
                        div {
                            class: format!(
                                "df-node-default{}{}",
                                if open { " open" } else { "" },
                                if data.foreign { " foreign" } else { "" },
                            ),
                            span {
                                class: "df-node-label",
                                "{ctx.node.label}"
                            }
                        }
                    }
                },

                Background {
                    variant: BackgroundVariant::Lines,
                    gap: 72.0,
                }
            }

            for key in opened() {
                {
                    let key_string = key.clone().into_string();
                    let focus_id = key.clone();
                    let close_id = key.clone();

                    rsx! {
                        Window {
                            key: "{key_string}",
                            entry: key,

                            onfocus: move |_| {
                                ui.focus(&focus_id);
                                select_node(nodes, Some(&focus_id));
                            },

                            onclose: move |_| {
                                ui.close(&close_id);
                                let active = ui.active();
                                select_node(nodes, active.as_ref());
                            },
                        }
                    }
                }
            }
        }
    }
}

fn select_node(mut nodes: Signal<Vec<Node<NodeData>>>, key: Option<&EntryId>) {
    let id = key.map(|key| key.clone().into_string());

    nodes.with_mut(|list| {
        for node in list {
            node.selected = id.as_deref() == Some(node.id.as_str()) && !node.data.foreign;
        }
    });
}
