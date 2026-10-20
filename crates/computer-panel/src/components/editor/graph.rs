use std::collections::HashSet;

use computer_model::{key::EntryKey, page::PageKind, value::Value};
use dioxus::prelude::*;
use dioxus_flow::{LayoutNode, compute_layout, prelude::*};

use crate::state::{app::AppState, ui::UiState};

const W: f64 = 160.0;
const H: f64 = 49.0;

#[derive(Clone, PartialEq)]
pub struct NodeData {
    pub key: EntryKey,
    pub foreign: bool,
}

#[derive(Default)]
pub struct Graph {
    pub nodes: Vec<Node<NodeData>>,
    pub edges: Vec<Edge>,
}

pub fn build() -> Graph {
    let app = consume_context::<AppState>();
    let ui = consume_context::<UiState>();

    let page_id = *ui.page.read();
    let pages = app.pages.read();
    let entries = app.entries.read();

    let Some(page) = page_id.and_then(|id| pages.get(&id)) else {
        return Graph::default();
    };

    let is_static = |key: &EntryKey| {
        pages
            .get(&key.page_id())
            .is_some_and(|p| p.kind == PageKind::Static)
    };

    let local: Vec<EntryKey> = page
        .entries
        .iter()
        .copied()
        .filter(|k| entries.contains_key(k))
        .collect();

    let mut links: Vec<(EntryKey, EntryKey)> = Vec::new();

    if page.kind != PageKind::Static {
        let mut seen = HashSet::new();

        for key in &local {
            let mut targets = Vec::new();
            entries[key]
                .fields
                .values()
                .for_each(|v| refs(v, &mut targets));

            for target in targets {
                if target == *key
                    || !entries.contains_key(&target)
                    || is_static(&target)
                    || !seen.insert((*key, target))
                {
                    continue;
                }
                links.push((*key, target));
            }
        }
    }

    let mut all = local.clone();
    let mut known: HashSet<EntryKey> = local.iter().copied().collect();
    for (_, target) in &links {
        if known.insert(*target) {
            all.push(*target);
        }
    }

    let layout = compute_layout(
        &all.iter()
            .map(|k| LayoutNode {
                id: k.to_string(),
                size: Size::new(W, H),
            })
            .collect::<Vec<_>>(),
        &links
            .iter()
            .map(|(s, t)| (s.to_string(), t.to_string()))
            .collect::<Vec<_>>(),
        &LayoutOptions::default().direction(LayoutDirection::LeftToRight),
    );

    let in_page: HashSet<EntryKey> = local.iter().copied().collect();

    let nodes = all
        .iter()
        .map(|key| {
            let id = key.to_string();
            let entry = &entries[key];
            let label = match entry.fields.get("name") {
                Some(Value::String(s)) if !s.is_empty() => s.clone(),
                _ => entry.kind.clone(),
            };
            let position = layout.get(&id).copied().unwrap_or(Point::ZERO);
            let foreign = !in_page.contains(key);

            Node::with_data(id, label, position, NodeData { key: *key, foreign })
                .size(Size::new(W, H))
                .sides(Side::Left, Side::Right)
                .selectable(!foreign)
                .draggable(false)
        })
        .collect();

    let edges = links
        .iter()
        .map(|(s, t)| {
            let mut edge = Edge::new(s.to_string(), t.to_string())
                .kind(EdgeKind::SmoothStep)
                .class(if s.page_id() == t.page_id() {
                    "same-page"
                } else {
                    "cross-page"
                });

            edge.selectable = false;
            edge
        })
        .collect();

    Graph { nodes, edges }
}

fn refs(value: &Value, out: &mut Vec<EntryKey>) {
    match value {
        Value::Reference(key) => out.push(*key),
        Value::Struct(fields) | Value::Variant { fields, .. } => {
            fields.values().for_each(|v| refs(v, out))
        }
        Value::List(items) => items.iter().for_each(|v| refs(v, out)),
        Value::Map(pairs) => pairs.iter().for_each(|(k, v)| {
            refs(k, out);
            refs(v, out);
        }),
        _ => {}
    }
}
