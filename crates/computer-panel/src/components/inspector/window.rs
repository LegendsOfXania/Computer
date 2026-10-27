use crate::components::utils::btn::Btn;
use crate::state::{app::AppState, ui::UiState};
use computer_model::{id::EntryId, value::Value};
use dioxus::{html::input_data::MouseButton, prelude::*};

#[component]
pub fn Window(entry: EntryId, onfocus: EventHandler<()>, onclose: EventHandler<()>) -> Element {
    let (state, ui) = (use_context::<AppState>(), use_context::<UiState>());
    let mut last = use_signal(|| None::<(i32, i32)>);

    let (x, y, z, active) = {
        let wins = ui.windows.read();

        let Some(i) = wins.iter().position(|w| w.id == entry) else {
            return rsx! {};
        };

        (wins[i].x, wins[i].y, 100 + i as i32, i + 1 == wins.len())
    };

    let label = state
        .entries
        .read()
        .get(&entry)
        .map(|entry| match entry.fields.get("name") {
            Some(Value::String(name)) if !name.is_empty() => name.clone(),
            _ => entry.kind.clone(),
        })
        .unwrap_or_default();

    rsx! {
        div {
            class: if active { "window active" } else { "window" },
            style: "left: {x}px; top: {y}px; z-index: {z};",
            onpointerdown: move |_| onfocus.call(()),

            div {
                class: "window-bar",

                onpointerdown: move |event| {
                    if event.trigger_button() == Some(MouseButton::Primary) {
                        let p = event.client_coordinates();
                        last.set(Some((p.x as i32, p.y as i32)));
                    }
                },

                onpointermove: move |event| {
                    let Some((lx, ly)) = *last.read() else {
                        return;
                    };

                    if !event.held_buttons().contains(MouseButton::Primary) {
                        last.set(None);
                        return;
                    }

                    let p = event.client_coordinates();
                    let (x, y) = (p.x as i32, p.y as i32);

                    ui.move_by(
                        &entry,
                        x.saturating_sub(lx),
                        y.saturating_sub(ly),
                    );

                    last.set(Some((x, y)));
                },

                onpointerup: move |_| last.set(None),
                onpointercancel: move |_| last.set(None),

                span { class: "window-title", "{label}" }

                span {
                    onpointerdown: move |event| event.stop_propagation(),
                    Btn {
                        onclick: move |_| onclose.call(())
                    }
                }
            }
        }
    }
}
