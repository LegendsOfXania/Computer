use std::collections::{HashMap, HashSet, VecDeque};
use dioxus::prelude::*;
use computer_model::{key::EntryKey, page::PageKind, value::Value};

use crate::state::AppState;

pub const NODE_W: f64 = 150.0;
pub const NODE_H: f64 = 56.0;

const GAP_X: f64 = 16.0;
const GAP_Y: f64 = 16.0;
const RANK_X: f64 = 260.0;
const ROW_Y: f64 = 100.0;

pub type Positions = HashMap<EntryKey, (f64, f64)>;
pub type Edges = Vec<(EntryKey, EntryKey)>;

pub fn compute(kind: PageKind, entries: &[EntryKey], app_state: &AppState, available_width: f64) -> (Positions, Edges) {
    match kind {
        PageKind::Static => (layout_static(entries, available_width), Vec::new()),
        PageKind::Sequence => layout_sequence(entries, app_state),
    }
}

fn layout_static(entries: &[EntryKey], available_width: f64) -> Positions {
    let width = available_width.max(NODE_W);
    let mut positions = Positions::with_capacity(entries.len());
    let (mut x, mut y) = (0.0, 0.0);

    for key in entries {
        if x > 0.0 && x + NODE_W > width {
            x = 0.0;
            y += NODE_H + GAP_Y;
        }
        positions.insert(*key, (x, y));
        x += NODE_W + GAP_X;
    }

    positions
}

fn layout_sequence(entries: &[EntryKey], app_state: &AppState) -> (Positions, Edges) {
    let local: HashSet<EntryKey> = entries.iter().copied().collect();
    let is_static = |key: &EntryKey| {
        app_state
            .pages
            .read()
            .get(&key.page_id())
            .map(|p| p.kind == PageKind::Static)
            .unwrap_or(false)
    };

    let refs: HashMap<EntryKey, (Vec<EntryKey>, Vec<EntryKey>)> = entries
        .iter()
        .map(|key| {
            let (local_refs, external_refs) = entry_references(app_state, key)
                .into_iter()
                .filter(|r| !is_static(r))
                .partition(|r| local.contains(r));
            (*key, (local_refs, external_refs))
        })
        .collect();

    let rank = topological_rank(entries, &refs);
    let mut positions = column_positions(entries, &rank);
    place_external_nodes(&refs, &rank, &mut positions);

    let edges = entries
        .iter()
        .flat_map(|key| {
            let (local_refs, external_refs) = &refs[key];
            local_refs.iter().chain(external_refs).map(move |target| (*key, *target))
        })
        .filter(|(_, target)| positions.contains_key(target))
        .collect();

    (positions, edges)
}

fn topological_rank(entries: &[EntryKey], refs: &HashMap<EntryKey, (Vec<EntryKey>, Vec<EntryKey>)>) -> HashMap<EntryKey, usize> {
    let mut indegree: HashMap<EntryKey, usize> = entries.iter().map(|k| (*k, 0)).collect();
    for (local_refs, _) in refs.values() {
        for target in local_refs {
            *indegree.entry(*target).or_insert(0) += 1;
        }
    }

    let mut rank = HashMap::new();
    let mut queue: VecDeque<EntryKey> = entries
        .iter()
        .copied()
        .filter(|k| indegree[k] == 0)
        .inspect(|k| {
            rank.insert(*k, 0);
        })
        .collect();

    while let Some(id) = queue.pop_front() {
        let current = rank[&id];
        for target in &refs[&id].0 {
            let r = rank.entry(*target).or_insert(0);
            *r = (*r).max(current + 1);
            let left = indegree.get_mut(target).unwrap();
            *left -= 1;
            if *left == 0 {
                queue.push_back(*target);
            }
        }
    }

    for key in entries {
        rank.entry(*key).or_insert(0);
    }
    rank
}

fn column_positions(entries: &[EntryKey], rank: &HashMap<EntryKey, usize>) -> Positions {
    let mut column_size: HashMap<usize, usize> = HashMap::new();
    for r in rank.values() {
        *column_size.entry(*r).or_insert(0) += 1;
    }

    let mut seen: HashMap<usize, usize> = HashMap::new();
    entries
        .iter()
        .map(|key| {
            let r = rank[key];
            let idx = *seen.entry(r).and_modify(|i| *i += 1).or_insert(0);
            let y = idx as f64 * ROW_Y - (column_size[&r] as f64 - 1.0) * ROW_Y / 2.0;
            (*key, (r as f64 * RANK_X, y))
        })
        .collect()
}

fn place_external_nodes(refs: &HashMap<EntryKey, (Vec<EntryKey>, Vec<EntryKey>)>, rank: &HashMap<EntryKey, usize>, positions: &mut Positions) {
    let external_x = (rank.values().copied().max().unwrap_or(0) as f64 + 1.0) * RANK_X;

    let mut external_keys: Vec<EntryKey> = refs.values().flat_map(|(_, ext)| ext.iter().copied()).collect();
    external_keys.sort();
    external_keys.dedup();

    for (idx, key) in external_keys.into_iter().enumerate() {
        positions.entry(key).or_insert((external_x, idx as f64 * ROW_Y));
    }
}

fn entry_references(app_state: &AppState, key: &EntryKey) -> Vec<EntryKey> {
    let mut result = Vec::new();
    
    if let Some(entry) = app_state.entries.read().get(key) {
        let mut stack: Vec<&Value> = entry.fields.values().collect();
        
        while let Some(value) = stack.pop() {
            match value {
                Value::Reference(k) => result.push(*k),
                Value::List(items) => stack.extend(items.iter()),
                Value::Struct(fields) => stack.extend(fields.values()),
                _ => {}
            }
        }
    }
    
    result
}