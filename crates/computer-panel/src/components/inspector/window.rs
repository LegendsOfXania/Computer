use computer_model::{key::EntryKey, value::Value};
use dioxus::{html::input_data::MouseButton, prelude::*};

use crate::components::utils::btn::Btn;
use crate::state::{app::AppState, ui::UiState};

#[component]
pub fn Window(entry: EntryKey, onfocus: EventHandler<()>, onclose: EventHandler<()>) -> Element {
    let state = use_context::<AppState>();
    let ui = use_context::<UiState>();
    let mut last = use_signal(|| None::<(i32, i32)>);

    let (x, y, z, active) = {
        let wins = ui.windows.read();
        let Some(i) = wins.iter().position(|w| w.key == entry) else {
            return rsx! {};
        };
        (wins[i].x, wins[i].y, 100 + i as i32, i + 1 == wins.len())
    };

    let label = {
        let entries = state.entries.read();
        let Some(e) = entries.get(&entry) else {
            return rsx! {};
        };
        match e.fields.get("name") {
            Some(Value::String(s)) if !s.is_empty() => s.clone(),
            _ => e.kind.clone(),
        }
    };

    let on_move = move |event: PointerEvent| {
        let Some((lx, ly)) = last() else { return };
        if !event.held_buttons().contains(MouseButton::Primary) {
            last.set(None);
            return;
        }
        let p = event.client_coordinates();
        let (px, py) = (p.x as i32, p.y as i32);
        ui.move_by(entry, px - lx, py - ly);
        last.set(Some((px, py)));
    };

    let on_end = move |_: PointerEvent| last.set(None);

    rsx! {
        div {
            class: if active { "window active" } else { "window" },
            style: "left: {x}px; top: {y}px; z-index: {z};",
            onpointerdown: move |_| onfocus.call(()),

            div { class: "window-bar",
                onpointerdown: move |event| {
                    if event.trigger_button() != Some(MouseButton::Primary) {
                        return;
                    }
                    let p = event.client_coordinates();
                    last.set(Some((p.x as i32, p.y as i32)));
                },
                onpointermove: on_move,
                onpointerup: on_end,
                onpointercancel: on_end,

                input { class: "window-title",
                    value: "{label}",
                    onpointerdown: move |event| {
                        event.stop_propagation();
                        onfocus.call(());
                    },
                    //todo check !empty => error toast
                }
                span { onpointerdown: move |event| event.stop_propagation(),
                    Btn { onclick: move |_| onclose.call(()) }
                }
            }
        }

        if last().is_some() {
            div {
                class: "window-drag",
                onpointermove: on_move,
                onpointerup: on_end,
                onpointercancel: on_end,
            }
        }
    }
}
