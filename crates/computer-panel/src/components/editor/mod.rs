mod graph;

use std::collections::HashSet;

use dioxus::prelude::*;
use dioxus_flow::prelude::*;

use crate::state::ui::UiState;
use graph::{build, NodeData};

#[component]
pub fn Editor() -> Element {
    let ui = use_context::<UiState>();
    let mut opened_entry = ui.opened_entry;

    let mut nodes = use_signal(Vec::<Node<NodeData>>::new);
    let mut edges = use_signal(Vec::<Edge>::new);
    let mut shown = use_signal(|| None::<u64>);
    let handle = use_flow_handle::<NodeData>();

    use_effect(move || {
        let page_id = *ui.opened_page.read();
        let graph = build();

        let selected: HashSet<String> = nodes
            .peek()
            .iter()
            .filter(|n| n.selected)
            .map(|n| n.id.clone())
            .collect();

        let mut next = graph.nodes;
        for node in &mut next {
            node.selected = selected.contains(&node.id);
        }

        nodes.set(next);
        edges.set(graph.edges);

        if *shown.peek() != page_id {
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
                    let key = nodes.read().iter().find(|n| n.id == id).map(|n| n.data.key);
                    if let Some(key) = key {
                        opened_entry.set(Some(key));
                    }
                },

                node_view: move |ctx: NodeViewCtx<NodeData>| {
                    let d = &ctx.node.data;
                    let open = *opened_entry.read() == Some(d.key);
                    let class = format!(
                        "df-node-default{}{}{}",
                        if d.linked { " linked" } else { "" },
                        if open { " open" } else { "" },
                        if d.foreign { " foreign" } else { "" },
                    );

                    rsx! {
                        div { class,
                            span { class: "df-node-label", "{ctx.node.label}" }
                        }
                    }
                },

                Background { variant: BackgroundVariant::Lines, gap: 72.0 }
            }
        }
    }
}