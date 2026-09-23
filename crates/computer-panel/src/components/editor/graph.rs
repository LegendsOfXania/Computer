use std::collections::HashMap;

use computer_model::{key::EntryKey, value::Value};
use dioxus::prelude::*;

use crate::{
    i18n::use_i18n,
    state::AppState,
};

const NODE_W: f32 = 150.0;
const NODE_H: f32 = 56.0;
const X: f32 = 260.0;
const Y: f32 = 100.0;
const COLS: usize = 4;

fn layout_static(entries: &[EntryKey]) -> HashMap<EntryKey, (f32, f32)> {
    entries
        .iter()
        .enumerate()
        .map(|(i, key)| {
            let (col, row) = (i % COLS, i / COLS);
            let x = col as f32 * X;
            let y = row as f32 * Y;
            (*key, (x, y))
        })
        .collect()
}

#[component]
pub fn Graph(entries: Vec<EntryKey>, links: bool) -> Element {
    let i18n = use_i18n();
    let mut selected = use_signal::<Option<EntryKey>>(|| entries.first().copied());
    let mut pan = use_signal(|| (0.0_f32, 0.0_f32));
    let mut dragging = use_signal(|| false);
    let mut last_mouse = use_signal(|| (0.0_f32, 0.0_f32));

    let positions = layout_static(&entries);

    let (width, height) = if entries.is_empty() {
        (0.0, 0.0)
    } else {
        positions.values().fold((0.0_f32, 0.0_f32), |(w, h), (x, y)| {
            (w.max(x + NODE_W), h.max(y + NODE_H))
        })
    };

    let (pan_x, pan_y) = *pan.read();
    let tx = pan_x;
    let ty = pan_y;
    
    let positions_for_effect = positions.clone();

    // Auto-pan when selection changes - center the selection in the viewport
    use_effect(move || {
        let Some(sel_key) = *selected.read() else { return };
        let Some(&(sel_x, sel_y)) = positions_for_effect.get(&sel_key) else { return };
        
        // Approximate viewport size - use a reasonable default
        // This will be refined when we have access to actual viewport dimensions
        let vw = 800.0; // Default viewport width approximation
        let vh = 600.0; // Default viewport height approximation
        
        let margin = 20.0;
        let mut p = pan.write();

        // Calculate visible area bounds
        let visible_left = p.0;
        let visible_right = p.0 + vw;
        let visible_top = p.1;
        let visible_bottom = p.1 + vh;

        // Check if selection is outside visible area (with margin) or near edges
        let needs_adjust_x = sel_x < visible_left + margin 
            || sel_x + NODE_W > visible_right - margin;
        let needs_adjust_y = sel_y < visible_top + margin 
            || sel_y + NODE_H > visible_bottom - margin;
        
        if needs_adjust_x || needs_adjust_y {
            // Center the selection in the viewport
            if needs_adjust_x {
                p.0 = sel_x - (vw - NODE_W) / 2.0;
            }
            if needs_adjust_y {
                p.1 = sel_y - (vh - NODE_H) / 2.0;
            }
            
            // Clamp pan to canvas bounds
            p.0 = p.0.clamp(0.0, (width - vw).max(0.0));
            p.1 = p.1.clamp(0.0, (height - vh).max(0.0));
        }
    });

    let entries_for_move = entries.clone();
    let mut move_selection = move |dc: i32, dr: i32| {
        let Some(cur) = *selected.read() else { return };
        let Some(idx) = entries_for_move.iter().position(|k| *k == cur) else { return };
        let (col, row) = (idx % COLS, idx / COLS);
        let (ncol, nrow) = (col as i32 + dc, row as i32 + dr);
        if ncol < 0 || nrow < 0 {
            return;
        }
        let nidx = nrow as usize * COLS + ncol as usize;
        if let Some(next) = entries_for_move.get(nidx) {
            selected.set(Some(*next));
        }
    };

    let empty_key = "editor.no_entries";
    rsx! {
        if entries.is_empty() {
            div {
                class: "graph-empty",
                "{i18n.t(empty_key)}"
            }
        } else {
            div {
                class: if *dragging.read() { "graph-viewport dragging" } else { "graph-viewport" },
                tabindex: "0",

                onmousedown: move |evt| {
                    dragging.set(true);
                    let c = evt.client_coordinates();
                    last_mouse.set((c.x as f32, c.y as f32));
                },
                onmouseup: move |_| dragging.set(false),
                onmouseleave: move |_| dragging.set(false),
                onmousemove: move |evt| {
                    if *dragging.read() {
                        let (lx, ly) = *last_mouse.read();
                        let c = evt.client_coordinates();
                        let (x, y) = (c.x as f32, c.y as f32);
                        let mut p = pan.write();
                        p.0 += x - lx;
                        p.1 += y - ly;
                        last_mouse.set((x, y));
                    }
                },
                onkeydown: move |evt| {
                    match evt.key() {
                        Key::ArrowUp => move_selection(0, -1),
                        Key::ArrowDown => move_selection(0, 1),
                        Key::ArrowLeft => move_selection(-1, 0),
                        Key::ArrowRight => move_selection(1, 0),
                        _ => {}
                    }
                },

                div {
                    class: if *dragging.read() { "graph-canvas dragging" } else { "graph-canvas" },
                    style: "width: {width}px; height: {height}px; transform: translate({tx}px, {ty}px);",

                    for key in entries.iter().copied() {
                        Node {
                            key: "{key}",
                            entry_key: key,
                            pos: positions[&key],
                            selected,
                        }
                    }

                    if links {

                    }
                }
            }
        }
    }
}

#[component]
fn Node(entry_key: EntryKey, pos: (f32, f32), mut selected: Signal<Option<EntryKey>>) -> Element {
    let state = use_context::<AppState>();
    let (x, y) = pos;
    let is_selected = *selected.read() == Some(entry_key);

    let name = state
        .entries
        .read()
        .get(&entry_key)
        .and_then(|e| e.fields.get("name"))
        .and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_else(|| entry_key.to_string());

    rsx! {
        div {
            class: if is_selected { "node selected" } else { "node" },
            style: "left: {x}px; top: {y}px; width: {NODE_W}px; height: {NODE_H}px;",
            onclick: move |evt| {
                evt.stop_propagation();
                selected.set(Some(entry_key));
            },
            span { class: "node-name", "{name}" }
            span { class: "node-key", "{entry_key}" }
        }
    }
}