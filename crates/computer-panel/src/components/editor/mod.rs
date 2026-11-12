mod graph;

use dioxus::prelude::*;
use dioxus_flow::prelude::*;
use std::collections::HashMap;

use crate::{
    components::{inspector::Inspector, utils::icon::Icon},
    state::{app::AppState, ui::UiState},
};
use graph::{NodeData, build, layout};

#[component]
pub fn Editor() -> Element {
    let (app, mut ui) = (use_context::<AppState>(), use_context::<UiState>());

    let mut nodes = use_signal(Vec::<Node<NodeData>>::new);
    let mut edges = use_signal(Vec::<Edge>::new);
    let handle = use_flow_handle::<NodeData>();

    let graph = use_memo(build);

    use_effect(move || {
        let _ = ui.get_opened_page();
        handle.set_viewport(Viewport::new(50.0, 60.0, 1.0), 0);
    });

    use_effect(move || {
        let graph = graph.read();
        let selected = ui.get_opened_entry();

        let previous: HashMap<String, _> = nodes
            .peek()
            .iter()
            .map(|node| (node.id.clone(), (node.position, node.measured)))
            .collect();

        let next: Vec<_> = graph
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
            .collect();

        if selected.is_some() && !next.iter().any(|node| node.selected) {
            ui.set_opened_entry(&app, None);
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

        let current_nodes = nodes.read().clone();
        let current_edges = edges.read().clone();

        if let Some(moved) = layout(&current_nodes, &current_edges)
            && !moved.is_empty()
        {
            nodes.with_mut(|nodes| {
                for (index, position) in moved {
                    nodes[index].position = position;
                }
            });
        }
    });

    rsx! {
        Stylesheet { href: asset!("/assets/style/editor/mod.css") }
        Stylesheet { href: asset!("/assets/style/editor/node.css") }
        Stylesheet { href: asset!("/assets/style/editor/edge.css") }

        div { class: "editor",
            Flow {
                nodes,
                edges,
                handle,
                fit_view: false,
                min_zoom: 0.5,
                max_zoom: 1.5,

                on_node_click: move |id: String| {
                    let Some(key) = nodes
                        .read()
                        .iter()
                        .find(|node| node.id == id)
                        .map(|node| node.data.key.clone())
                    else {
                        return;
                    };

                    ui.set_opened_entry(&app, Some(key));
                },

                on_pane_click: move |_| {
                    ui.set_opened_entry(&app, None);
                },

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
