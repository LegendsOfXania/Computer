mod graph;

use computer_model::id::PageId;
use dioxus::prelude::*;
use dioxus_flow::prelude::*;
use std::collections::HashSet;

use crate::{components::utils::icon::Icon, state::ui::UiState};
use graph::{NodeData, build, layout};

#[component]
pub fn Editor() -> Element {
    let ui = use_context::<UiState>();
    let mut opened_entry = ui.opened_entry;

    let mut nodes = use_signal(Vec::<Node<NodeData>>::new);
    let mut edges = use_signal(Vec::<Edge>::new);
    let mut shown = use_signal(|| None::<PageId>);
    let handle = use_flow_handle::<NodeData>();

    use_effect(move || {
        let page_id = ui.opened_page.read().clone();
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
            opened_entry.set(None);
        }
    });

    use_effect(move || {
        let current = nodes.read().clone();
        let current_edges = edges.read().clone();
        let mut next = current.clone();

        if layout(&mut next, &current_edges) {
            nodes.set(next);
            handle.fit_view(250);
        }
    });

    rsx! {
        document::Stylesheet { href: asset!("/assets/style/editor/mod.css") }
        document::Stylesheet { href: asset!("/assets/style/editor/node.css") }
        document::Stylesheet { href: asset!("/assets/style/editor/edge.css") }

        div {
            style: "width: 100%; height: 100%;",

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
                        opened_entry.set(Some(key));
                    }
                },

                node_view: move |ctx: NodeViewCtx<NodeData>| {
                    rsx! {
                        div {
                            class: if ctx.node.data.foreign {
                                "df-node-default foreign"
                            } else {
                                "df-node-default"
                            },

                            Icon { name: ctx.node.data.icon.clone() }

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
        }
    }
}
