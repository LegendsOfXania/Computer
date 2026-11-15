use crate::state::{app::AppState, ui::UiState};
use computer_model::{
    id::{EntryId, PageId},
    page::PageKind,
};
use dioxus::prelude::*;
use dioxus_flow::{LayoutNode, compute_layout, prelude::*};
use indexmap::{IndexMap, IndexSet};
use std::collections::{HashMap, HashSet};

#[derive(Clone, PartialEq)]
pub struct NodeData {
    pub key: EntryId,
    pub foreign: Option<PageId>,
    pub icon: String,
}

#[derive(Default, PartialEq)]
pub struct Graph {
    pub nodes: Vec<Node<NodeData>>,
    pub edges: Vec<Edge>,
}

pub fn build() -> Graph {
    let (app, ui) = (consume_context::<AppState>(), consume_context::<UiState>());
    let library = app.library.read();

    let Some(page) = ui
        .get_opened_page()
        .and_then(|id| library.pages.get(&id))
    else {
        return Graph::default();
    };

    let owners: HashMap<EntryId, (PageId, bool)> = library
        .pages
        .iter()
        .flat_map(|(id, origin)| {
            let is_static = origin.kind == PageKind::Static;

            origin
                .entries
                .iter()
                .map(move |entry| (*entry, (*id, is_static)))
        })
        .collect();

    let local: Vec<EntryId> = page
        .entries
        .iter()
        .copied()
        .filter(|id| library.entries.contains_key(id))
        .collect();

    let in_page: HashSet<EntryId> = local.iter().copied().collect();

    let mut links: Vec<(EntryId, EntryId)> = Vec::new();

    if page.kind != PageKind::Static {
        for &source in &local {
            let Some(entry) = library.entries.get(&source) else {
                continue;
            };

            let targets: IndexSet<EntryId> = entry
                .fields
                .values()
                .flat_map(|value| value.references())
                .collect();

            links.extend(
                targets
                    .into_iter()
                    .filter(|target| {
                        *target != source
                            && library.entries.contains_key(target)
                            && owners.get(target).is_some_and(|&(_, is_static)| !is_static)
                    })
                    .map(|target| (source, target)),
            );
        }
    }

    let foreign: IndexMap<EntryId, PageId> = links
        .iter()
        .filter(|(_, target)| !in_page.contains(target))
        .filter_map(|(_, target)| Some((*target, owners.get(target)?.0)))
        .collect();

    let nodes = local
        .iter()
        .map(|&key| (key, None))
        .chain(
            foreign
                .into_iter()
                .map(|(key, origin)| (key, Some(origin))),
        )
        .filter_map(|(key, origin)| {
            let entry = library.entries.get(&key)?;
            let (icon, color) = app.look_of(&entry.kind);

            Some(
                Node::with_data(
                    key.to_string(),
                    entry.label(),
                    Point::ZERO,
                    NodeData {
                        key,
                        foreign: origin,
                        icon,
                    },
                )
                .style(format!("--node-color: {color};"))
                .sides(Side::Left, Side::Right)
                .selectable(origin.is_none())
                .draggable(false),
            )
        })
        .collect();

    let edges = links
        .into_iter()
        .map(|(source, target)| {
            let color = library
                .entries
                .get(&source)
                .map(|entry| app.look_of(&entry.kind).1)
                .filter(|color| !color.is_empty())
                .unwrap_or_else(|| "var(--faint)".to_string());

            let mut edge =
                Edge::new(source.to_string(), target.to_string()).kind(EdgeKind::SmoothStep);

            if !in_page.contains(&target) {
                edge = edge.class("cross-page");
            }

            edge.style = Some(format!("stroke: {color};"));
            edge.selectable = false;
            edge
        })
        .collect();

    Graph { nodes, edges }
}

pub fn layout(nodes: &[Node<NodeData>], edges: &[Edge]) -> Option<Vec<(usize, Point)>> {
    let layout_nodes = nodes
        .iter()
        .map(|node| {
            Some(LayoutNode {
                id: node.id.clone(),
                size: node.measured?,
            })
        })
        .collect::<Option<Vec<_>>>()
        .filter(|nodes| !nodes.is_empty())?;

    let layout_edges: Vec<_> = edges
        .iter()
        .map(|edge| (edge.source.clone(), edge.target.clone()))
        .collect();

    let mut positions = compute_layout(
        &layout_nodes,
        &layout_edges,
        &LayoutOptions::default().direction(LayoutDirection::LeftToRight),
    );

    let min_x = positions
        .values()
        .map(|p| p.x)
        .fold(f64::INFINITY, f64::min);
    let min_y = positions
        .values()
        .map(|p| p.y)
        .fold(f64::INFINITY, f64::min);

    for position in positions.values_mut() {
        position.x -= min_x;
        position.y -= min_y;
    }

    Some(
        nodes
            .iter()
            .enumerate()
            .filter_map(|(index, node)| {
                let position = *positions.get(&node.id)?;
                (node.position != position).then_some((index, position))
            })
            .collect(),
    )
}
