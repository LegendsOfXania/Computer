mod graph;

use computer_model::id::PageId;
use dioxus::prelude::*;
use dioxus_flow::prelude::*;
use std::collections::HashMap;

use crate::{
    components::{inspector::Inspector, utils::icon::Icon},
    state::ui::UiState,
};
use graph::{NodeData, build, layout};

#[component]
pub fn Editor() -> Element {
    let mut ui = use_context::<UiState>();
    let mut opened_entry = ui.opened_entry;

    let mut nodes = use_signal(Vec::<Node<NodeData>>::new);
    let mut edges = use_signal(Vec::<Edge>::new);
    let mut shown = use_signal(|| None::<PageId>);
    let mut needs_fit = use_signal(|| false);
    let handle = use_flow_handle::<NodeData>();

    let graph = use_memo(build);

    use_effect(move || {
        let graph = graph.read();
        let page_id = ui.opened_page.read().clone();
        let selected = opened_entry.read().clone();

        let next: Vec<_> = {
            let old = nodes.peek();
            let previous: HashMap<&str, _> = old
                .iter()
                .map(|node| (node.id.as_str(), (node.position, node.measured)))
                .collect();

            graph
                .nodes
                .iter()
                .cloned()
                .map(|mut node| {
                    if let Some(&(position, measured)) = previous.get(node.id.as_str()) {
                        node.position = position;
                        node.measured = measured;
                    }

                    node.selected =
                        node.data.foreign.is_none() && selected.as_ref() == Some(&node.data.key);

                    node
                })
                .collect()
        };

        if shown.peek().as_ref() != page_id.as_ref() {
            shown.set(page_id);
            needs_fit.set(true);

            if selected.is_some() && !next.iter().any(|node| node.selected) {
                opened_entry.set(None);
            }
        }

        if *nodes.peek() != next {
            nodes.set(next);
        }

        if *edges.peek() != graph.edges {
            edges.set(graph.edges.clone());
        }
    });

    let layout_key = use_memo(move || {
        let sizes = nodes
            .read()
            .iter()
            .map(|node| node.measured.map(|size| (node.id.clone(), size)))
            .collect::<Option<Vec<_>>>()?;

        let links = edges
            .read()
            .iter()
            .map(|edge| (edge.source.clone(), edge.target.clone()))
            .collect::<Vec<_>>();

        Some((sizes, links))
    });

    use_effect(move || {
        if layout_key.read().is_none() {
            return;
        }

        let fit = *needs_fit.read();

        let moved = layout(&nodes.peek(), &edges.peek());

        if let Some(moved) = moved {
            if !moved.is_empty() {
                nodes.with_mut(|nodes| {
                    for (index, position) in moved {
                        nodes[index].position = position;
                    }
                });
            }
        }

        if fit {
            needs_fit.set(false);
            handle.fit_view(250);
        }
    });

    rsx! {
        document::Stylesheet { href: asset!("/assets/style/editor/mod.css") }
        document::Stylesheet { href: asset!("/assets/style/editor/node.css") }
        document::Stylesheet { href: asset!("/assets/style/editor/edge.css") }

        div { class: "editor",

            Flow {
                nodes,
                edges,
                handle,
                fit_view: true,

                on_node_click: move |id: String| {
                    let Some((key, origin)) = nodes
                        .read()
                        .iter()
                        .find(|node| node.id == id)
                        .map(|node| (node.data.key.clone(), node.data.foreign.clone()))
                    else {
                        return;
                    };

                    if let Some(page_id) = origin {
                        ui.opened_page.set(Some(page_id));
                    }

                    opened_entry.set(Some(key));
                },

                on_pane_click: move |_| opened_entry.set(None),

                node_view: move |ctx: NodeViewCtx<NodeData>| {
                    rsx! {
                        div {
                            class: if ctx.node.data.foreign.is_some() {
                                "df-node-default foreign"
                            } else {
                                "df-node-default"
                            },

                            Icon { name: ctx.node.data.icon.clone() }

                            span { class: "df-node-label", "{ctx.node.label}" }
                        }
                    }
                },

                Background {
                    variant: BackgroundVariant::Lines,
                    gap: 72.0,
                }
            }

            Inspector {}
        }
    }
}
