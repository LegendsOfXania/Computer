use dioxus::prelude::*;

use computer_model::{key::EntryKey, page::PageKind};

use crate::{components::editor::layout::Edges, i18n::use_i18n, nav::Nav, state::AppState};

use super::{layout, layout::Positions, node::EntryNode};

const MIN_ZOOM: f64 = 0.3;
const MAX_ZOOM: f64 = 2.0;
const ZOOM_STEP: f64 = 1.15;

#[derive(Clone, Copy, PartialEq, Debug)]
struct Camera {
    x: f64,
    y: f64,
    zoom: f64,
}

impl Default for Camera {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, zoom: 1.0 }
    }
}

impl Camera {
    fn zoomed(self, factor: f64, anchor: (f64, f64)) -> Camera {
        let zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
        let world = ((anchor.0 - self.x) / self.zoom, (anchor.1 - self.y) / self.zoom);
        Camera { x: anchor.0 - world.0 * zoom, y: anchor.1 - world.1 * zoom, zoom }
    }

    fn centered_on(pos: (f64, f64), container: (f64, f64)) -> Camera {
        Camera {
            x: container.0 / 2.0 - (pos.0 + layout::NODE_W / 2.0),
            y: container.1 / 2.0 - (pos.1 + layout::NODE_H / 2.0),
            zoom: 1.0,
        }
    }

    fn screen(self, pos: (f64, f64)) -> (f64, f64) {
        (pos.0 * self.zoom + self.x, pos.1 * self.zoom + self.y)
    }
}

fn is_visible(cam: Camera, container: (f64, f64), pos: (f64, f64)) -> bool {
    let (sx, sy) = cam.screen(pos);
    let (w, h) = (layout::NODE_W * cam.zoom, layout::NODE_H * cam.zoom);
    sx + w >= 0.0 && sx <= container.0 && sy + h >= 0.0 && sy <= container.1
}

fn nearest(positions: &Positions, from: EntryKey, dir: (f64, f64)) -> Option<EntryKey> {
    let (cx, cy) = *positions.get(&from)?;
    positions
        .iter()
        .filter(|(k, _)| **k != from)
        .filter_map(|(k, &(x, y))| {
            let (dx, dy) = (x - cx, y - cy);
            let along = dx * dir.0 + dy * dir.1;
            (along > 0.0).then(|| (*k, along + (dx * dir.1 - dy * dir.0).abs() * 2.0))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(k, _)| k)
}

#[component]
pub fn Graph(entries: Vec<EntryKey>, kind: PageKind, nav: Nav) -> Element {
    let i18n = use_i18n();
    let app_state = use_context::<AppState>();

    let mut container = use_signal(|| (0.0_f64, 0.0_f64));
    let mut camera = use_signal(Camera::default);
    let mut selected = use_signal(|| entries.first().copied());
    let mut drag_from = use_signal(|| None::<(f64, f64)>);

    let entries_for_layout = entries.clone();
    let app_state_for_layout = app_state;
    let layout_result = use_memo(move || {
        layout::compute(kind, &entries_for_layout, &app_state_for_layout, container.read().0)
    });

    let entries_for_mount = entries.clone();

    use_effect(move || {
        nav.on_key(Callback::new(move |(key, modifiers): (Key, Modifiers)| {
            let size = *container.read();
            let center = (size.0 / 2.0, size.1 / 2.0);

            if modifiers.ctrl() {
                let factor = match &key {
                    Key::Character(c) if c == "+" || c == "=" => ZOOM_STEP,
                    Key::Character(c) if c == "-" => 1.0 / ZOOM_STEP,
                    _ => return false,
                };
                let cam = *camera.read();
                camera.set(cam.zoomed(factor, center));
                return true;
            }

            let dir = match key {
                Key::ArrowUp => (0.0, -1.0),
                Key::ArrowDown => (0.0, 1.0),
                Key::ArrowLeft => (-1.0, 0.0),
                Key::ArrowRight => (1.0, 0.0),
                _ => return false,
            };

            let (positions, _) = &*layout_result.read();
            let Some(cur) = *selected.read() else { return false };
            let Some(next) = nearest(positions, cur, dir) else { return false };
            selected.set(Some(next));

            if let Some(&pos) = positions.get(&next) {
                if !is_visible(*camera.read(), size, pos) {
                    camera.set(Camera::centered_on(pos, size));
                }
            }

            true
        }));
    });

    rsx! {
        document::Stylesheet { href: asset!("/assets/style/editor/graph.css") }

        if entries.is_empty() {
            div { class: "graph-empty", "{i18n.t(\"editor.no_entries\")}" }
        } else {
            div {
                class: if drag_from.read().is_some() { "graph-viewport dragging" } else { "graph-viewport" },
                tabindex: "0",

                onmounted: move |e| {
                    let data = e.data();
                    let entries = entries_for_mount.clone();
                    let app_state = app_state_for_layout;
                    spawn(async move {
                        let Ok(rect) = data.get_client_rect().await else { return };
                        let size = (rect.size.width, rect.size.height);
                        container.set(size);
                        let (positions, _) = layout::compute(kind, &entries, &app_state, size.0);
                        if let Some(first) = entries.first() {
                            if let Some(&pos) = positions.get(first) {
                                camera.set(Camera::centered_on(pos, size));
                            }
                        }
                    });
                },

                onmousedown: move |e| {
                    let c = e.client_coordinates();
                    drag_from.set(Some((c.x, c.y)));
                },
                onmouseup: move |_| drag_from.set(None),

                onmouseleave: move |_| drag_from.set(None),

                onmousemove: move |e| {
                    let Some((lx, ly)) = *drag_from.read() else { return };
                    let c = e.client_coordinates();
                    drag_from.set(Some((c.x, c.y)));
                    camera
                        .with_mut(|cam| {
                            cam.x += c.x - lx;
                            cam.y += c.y - ly;
                        });
                },

                onwheel: move |e| {
                    e.prevent_default();
                    let c = e.client_coordinates();
                    let factor = if e.delta().strip_units().y < 0.0 {
                        ZOOM_STEP
                    } else {
                        1.0 / ZOOM_STEP
                    };
                    let cam = *camera.read();
                    camera.set(cam.zoomed(factor, (c.x, c.y)));
                },

                div {
                    class: "graph-canvas",
                    style: {
                        let cam = *camera.read();
                        format!("transform: translate({}px, {}px) scale({});", cam.x, cam.y, cam.zoom)
                    },

                    Edge {
                        edges: layout_result.read().1.clone(),
                        positions: layout_result.read().0.clone(),
                    }

                    for key in entries.iter().copied() {
                        if let Some(&pos) = layout_result.read().0.get(&key) {
                            EntryNode {
                                key: "{key}",
                                entry_key: key,
                                pos,
                                selected: *selected.read() == Some(key),
                                onselect: move |k| selected.set(Some(k)),
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn Edge(edges: Edges, positions: Positions) -> Element {
    rsx! {
        svg { class: "graph-edges",
            for (from , to) in edges.iter().copied() {
                if let (Some(&a), Some(&b)) = (positions.get(&from), positions.get(&to)) {
                    line {
                        x1: "{a.0 + layout::NODE_W / 2.0}",
                        y1: "{a.1 + layout::NODE_H / 2.0}",
                        x2: "{b.0 + layout::NODE_W / 2.0}",
                        y2: "{b.1 + layout::NODE_H / 2.0}",
                    }
                }
            }
        }
    }
}