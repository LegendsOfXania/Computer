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

    use_effect(move || {
        let page_id = ui.opened_page.read().clone();
        let graph = build();
        let selected = opened_entry.peek().clone();

        let next: Vec<_> = {
            let old = nodes.peek();
            let previous: HashMap<&str, _> = old
                .iter()
                .map(|node| (node.id.as_str(), (node.position, node.measured)))
                .collect();

            graph
                .nodes
                .into_iter()
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

        nodes.set(next);
        edges.set(graph.edges);
    });

    use_effect(move || {
        let Some(moved) = layout(&nodes.read(), &edges.read()) else {
            return;
        };

        if !moved.is_empty() {
            nodes.with_mut(|nodes| {
                for (index, position) in moved {
                    nodes[index].position = position;
                }
            });
        }

        if *needs_fit.peek() {
            needs_fit.set(false);
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

                Inspector {}
            }
        }
    }
}
