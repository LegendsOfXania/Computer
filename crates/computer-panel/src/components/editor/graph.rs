use crate::state::{app::AppState, ui::UiState};
use computer_model::{
    id::{EntryId, Identifier, PageId},
    page::PageKind,
    value::Value,
};
use dioxus::prelude::*;
use dioxus_flow::{LayoutNode, compute_layout, prelude::*};
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
    let pages = app.pages.read();
    let entries = app.entries.read();
    let registry = app.registry.read();

    let Some(page_id) = ui.get_opened_page() else {
        return Graph::default();
    };

    let Some(page) = pages.get(&page_id) else {
        return Graph::default();
    };

    let owners: HashMap<&EntryId, (&PageId, bool)> = pages
        .iter()
        .flat_map(|(id, origin)| {
            let is_static = origin.kind == PageKind::Static;
            origin
                .entries
                .iter()
                .map(move |entry| (entry, (id, is_static)))
        })
        .collect();

    let local: Vec<&EntryId> = page
        .entries
        .iter()
        .filter(|id| entries.contains_key(*id))
        .collect();

    let in_page: HashSet<&EntryId> = local.iter().copied().collect();

    let mut links: Vec<(&EntryId, EntryId)> = Vec::new();

    if page.kind != PageKind::Static {
        for &source in &local {
            let mut targets = HashSet::new();

            for value in entries[source].fields.values() {
                collect_refs(value, &mut targets);
            }

            links.extend(
                targets
                    .into_iter()
                    .filter(|target| {
                        target != source
                            && entries.contains_key(target)
                            && owners.get(target).is_some_and(|&(_, is_static)| !is_static)
                    })
                    .map(|target| (source, target)),
            );
        }
    }

    let foreign: HashMap<&EntryId, &PageId> = links
        .iter()
        .filter(|(_, target)| !in_page.contains(target))
        .map(|(_, target)| (target, owners[target].0))
        .collect();

    let nodes = local
        .iter()
        .map(|&key| (key, None))
        .chain(foreign.into_iter().map(|(key, origin)| (key, Some(origin))))
        .filter_map(|(key, origin)| {
            let entry = entries.get(key)?;
            let definition = registry.get(&entry.kind);

            let label = entry.label();

            let color = definition
                .map(|definition| definition.color.clone())
                .unwrap_or_default();

            let icon = definition
                .map(|definition| definition.icon.clone())
                .unwrap_or_default();

            Some(
                Node::with_data(
                    key.clone().into_string(),
                    label,
                    Point::ZERO,
                    NodeData {
                        key: key.clone(),
                        foreign: origin.cloned(),
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
            let color = entries
                .get(source)
                .and_then(|entry| registry.get(&entry.kind))
                .map(|definition| definition.color.as_str())
                .filter(|color| !color.is_empty())
                .unwrap_or("var(--faint)");

            let cross_page = !in_page.contains(&target);

            let mut edge = Edge::new(source.clone().into_string(), target.into_string())
                .kind(EdgeKind::SmoothStep);

            if cross_page {
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

fn collect_refs(value: &Value, out: &mut HashSet<EntryId>) {
    match value {
        Value::Reference(id) => {
            out.insert(id.clone());
        }
        Value::Struct(fields) | Value::Variant { fields, .. } => {
            fields.values().for_each(|value| collect_refs(value, out));
        }
        Value::List(items) => {
            items.iter().for_each(|value| collect_refs(value, out));
        }
        Value::Map(pairs) => {
            pairs.iter().for_each(|(key, value)| {
                collect_refs(key, out);
                collect_refs(value, out);
            });
        }
        _ => {}
    }
}
