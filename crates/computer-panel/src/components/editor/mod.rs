use dioxus::prelude::*;
use dioxus_flow::prelude::*;

#[component]
pub fn Editor() -> Element {
    let nodes = use_signal(|| {
        vec![
            Node::new("1", "Entry 1", (0.0, 0.0)).sides(Side::Left, Side::Right),
            Node::new("2", "Entry 2", (200.0, 0.0)).sides(Side::Left, Side::Right),
        ]
    });
    let edges = use_signal(|| {
        vec![Edge::new("1", "2")
            .kind(EdgeKind::SmoothStep)
            .marker_end(MarkerKind::None)]
    });

    rsx! {
        document::Stylesheet { href: asset!("/assets/style/editor/mod.css") }
        document::Stylesheet { href: asset!("/assets/style/editor/node.css") }

        div { style: "width: 100%; height: 100%;",
            Flow {
                nodes,
                edges,
                fit_view: true,

                node_view: move |ctx: NodeViewCtx<()>| {
                    let linked = edges.read().iter().any(|e| e.target == ctx.node.id);
                    rsx! {
                        div { class: if linked { "df-node-default linked" } else { "df-node-default" },
                            span { class: "df-node-label", "{ctx.node.label}" }
                        }
                    }
                },

                Background { variant: BackgroundVariant::Lines, gap: 72.0 }
            }
        }
    }
}