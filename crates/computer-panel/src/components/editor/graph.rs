use crate::state::{app::AppState, ui::UiState};
use computer_model::{
    id::{EntryId, Identifier},
    page::PageKind,
    value::Value,
};
use dioxus::prelude::*;
use dioxus_flow::{LayoutNode, compute_layout, prelude::*};
use std::collections::{HashMap, HashSet};

const W: f64 = 160.0;
const H: f64 = 49.0;

#[derive(Clone, PartialEq)]
pub struct NodeData {
    pub key: EntryId,
    pub foreign: bool,
}

#[derive(Default)]
pub struct Graph {
    pub nodes: Vec<Node<NodeData>>,
    pub edges: Vec<Edge>,
}

pub fn build() -> Graph {
    let (app, ui) = (consume_context::<AppState>(), consume_context::<UiState>());
    let (pages, entries) = (app.pages.read(), app.entries.read());

    let Some(page_id) = ui.page.read().as_ref().cloned() else {
        return Graph::default();
    };

    let Some(page) = pages.get(&page_id) else {
        return Graph::default();
    };

    let mut owners = HashMap::new();
    let mut static_entries = HashSet::new();

    for (page_id, page) in pages.iter() {
        for id in &page.entries {
            owners.insert(id.clone(), page_id.clone());

            if page.kind == PageKind::Static {
                static_entries.insert(id.clone());
            }
        }
    }

    let local: Vec<EntryId> = page
        .entries
        .iter()
        .filter(|id| entries.contains_key(*id))
        .cloned()
        .collect();

    let mut links = Vec::new();

    if page.kind != PageKind::Static {
        let mut seen = HashSet::new();

        for id in &local {
            let Some(entry) = entries.get(id) else {
                continue;
            };

            let mut targets = Vec::new();

            for value in entry.fields.values() {
                refs(value, &mut targets);
            }

            for target in targets {
                if target != *id
                    && entries.contains_key(&target)
                    && !static_entries.contains(&target)
                    && seen.insert((id.clone(), target.clone()))
                {
                    links.push((id.clone(), target));
                }
            }
        }
    }

    let mut all = local.clone();
    let mut known: HashSet<EntryId> = local.iter().cloned().collect();

    for (_, target) in &links {
        if known.insert(target.clone()) {
            all.push(target.clone());
        }
    }

    let layout_nodes: Vec<_> = all
        .iter()
        .map(|id| LayoutNode {
            id: id.clone().into_string(),
            size: Size::new(W, H),
        })
        .collect();

    let layout_edges: Vec<_> = links
        .iter()
        .map(|(source, target)| (source.clone().into_string(), target.clone().into_string()))
        .collect();

    let layout = compute_layout(
        &layout_nodes,
        &layout_edges,
        &LayoutOptions::default().direction(LayoutDirection::LeftToRight),
    );

    let in_page: HashSet<EntryId> = local.iter().cloned().collect();

    let nodes = all
        .iter()
        .filter_map(|key| {
            let entry = entries.get(key)?;
            let id = key.clone().into_string();

            let label = match entry.fields.get("name") {
                Some(Value::String(name)) if !name.is_empty() => name.clone(),
                _ => entry.kind.clone(),
            };

            let foreign = !in_page.contains(key);
            let position = layout.get(&id).copied().unwrap_or(Point::ZERO);

            Some(
                Node::with_data(
                    id,
                    label,
                    position,
                    NodeData {
                        key: key.clone(),
                        foreign,
                    },
                )
                .size(Size::new(W, H))
                .sides(Side::Left, Side::Right)
                .selectable(!foreign)
                .draggable(false),
            )
        })
        .collect();

    let edges = links
        .iter()
        .map(|(source, target)| {
            let same_page = owners
                .get(source)
                .zip(owners.get(target))
                .is_some_and(|(source_page, target_page)| source_page == target_page);

            let mut edge = Edge::new(source.clone().into_string(), target.clone().into_string())
                .kind(EdgeKind::SmoothStep)
                .class(if same_page { "same-page" } else { "cross-page" });

            edge.selectable = false;
            edge
        })
        .collect();

    Graph { nodes, edges }
}

fn refs(value: &Value, out: &mut Vec<EntryId>) {
    match value {
        Value::Reference(id) => out.push(id.clone()),

        Value::Struct(fields) | Value::Variant { fields, .. } => {
            fields.values().for_each(|value| refs(value, out));
        }

        Value::List(items) => {
            items.iter().for_each(|value| refs(value, out));
        }

        Value::Map(pairs) => {
            pairs.iter().for_each(|(key, value)| {
                refs(key, out);
                refs(value, out);
            });
        }

        _ => {}
    }
}
