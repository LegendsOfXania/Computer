use crate::state::{app::AppState, ui::UiState};
use computer_model::{
    id::{EntryId, Identifier},
    page::PageKind,
    value::Value,
};
use dioxus::prelude::*;
use dioxus_flow::{LayoutNode, compute_layout, prelude::*};
use std::collections::{HashMap, HashSet};

#[derive(Clone, PartialEq)]
pub struct NodeData {
    pub key: EntryId,
    pub foreign: bool,
    pub color: String,
    pub icon: String,
}

#[derive(Default)]
pub struct Graph {
    pub nodes: Vec<Node<NodeData>>,
    pub edges: Vec<Edge>,
}

pub fn build() -> Graph {
    let app = consume_context::<AppState>();
    let ui = consume_context::<UiState>();
    let pages = app.pages.read();
    let entries = app.entries.read();
    let registry = app.registry.read();

    let Some(page_id) = ui.opened_page.read().clone() else {
        return Graph::default();
    };

    let Some(page) = pages.get(&page_id) else {
        return Graph::default();
    };

    let mut owners = HashMap::new();
    let mut static_entries = HashSet::new();

    for (owner, page) in pages.iter() {
        for id in &page.entries {
            owners.insert(id.clone(), owner.clone());

            if page.kind == PageKind::Static {
                static_entries.insert(id.clone());
            }
        }
    }

    let local: Vec<_> = page
        .entries
        .iter()
        .filter(|id| entries.contains_key(*id))
        .cloned()
        .collect();

    let mut links = Vec::new();
    let mut seen_links = HashSet::new();

    if page.kind != PageKind::Static {
        for source in &local {
            let Some(entry) = entries.get(source) else {
                continue;
            };

            let mut targets = HashSet::new();

            for value in entry.fields.values() {
                collect_refs(value, &mut targets);
            }

            for target in targets {
                if target == *source
                    || static_entries.contains(&target)
                    || !entries.contains_key(&target)
                    || !seen_links.insert((source.clone(), target.clone()))
                {
                    continue;
                }

                links.push((source.clone(), target));
            }
        }
    }

    let mut all = local.clone();
    let mut known: HashSet<_> = local.iter().cloned().collect();

    for (_, target) in &links {
        if known.insert(target.clone()) {
            all.push(target.clone());
        }
    }

    let in_page: HashSet<_> = local.into_iter().collect();

    let nodes = all
        .iter()
        .filter_map(|key| {
            let entry = entries.get(key)?;
            let definition = registry.get(&entry.kind);

            let label = match entry.fields.get("name") {
                Some(Value::String(name)) if !name.is_empty() => name.clone(),
                _ => entry.kind.clone(),
            };

            let color = definition
                .map(|definition| definition.color.clone())
                .filter(|color| !color.is_empty())
                .unwrap_or_else(|| "var(--surface-2)".into());

            let icon = definition
                .map(|definition| definition.icon.clone())
                .unwrap_or_default();

            let foreign = !in_page.contains(key);

            Some(
                Node::with_data(
                    key.clone().into_string(),
                    label,
                    Point::ZERO,
                    NodeData {
                        key: key.clone(),
                        foreign,
                        color: color.clone(),
                        icon,
                    },
                )
                .style(format!("--node-color: {color};"))
                .sides(Side::Left, Side::Right)
                .selectable(!foreign)
                .draggable(false),
            )
        })
        .collect();

    let edges = links
        .into_iter()
        .map(|(source, target)| {
            let same_page = owners
                .get(&source)
                .zip(owners.get(&target))
                .is_some_and(|(source_page, target_page)| source_page == target_page);

            let color = entries
                .get(&source)
                .and_then(|entry| registry.get(&entry.kind))
                .map(|definition| definition.color.as_str())
                .filter(|color| !color.is_empty())
                .unwrap_or("var(--faint)");

            let mut edge = Edge::new(source.clone().into_string(), target.into_string())
                .kind(EdgeKind::SmoothStep)
                .class(if same_page { "same-page" } else { "cross-page" });

            edge.style = Some(format!("stroke: {color};"));
            edge.selectable = false;
            edge
        })
        .collect();

    Graph { nodes, edges }
}

pub fn layout(nodes: &mut [Node<NodeData>], edges: &[Edge]) -> bool {
    if nodes.is_empty() || nodes.iter().any(|node| node.measured.is_none()) {
        return false;
    }

    let layout_nodes: Vec<_> = nodes
        .iter()
        .filter_map(|node| {
            Some(LayoutNode {
                id: node.id.clone(),
                size: node.measured?,
            })
        })
        .collect();

    let layout_edges: Vec<_> = edges
        .iter()
        .map(|edge| (edge.source.clone(), edge.target.clone()))
        .collect();

    let positions = compute_layout(
        &layout_nodes,
        &layout_edges,
        &LayoutOptions::default().direction(LayoutDirection::LeftToRight),
    );

    let mut changed = false;

    for node in nodes {
        if let Some(position) = positions.get(&node.id) {
            if node.position != *position {
                node.position = *position;
                changed = true;
            }
        }
    }

    changed
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
