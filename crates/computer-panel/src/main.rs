use dioxus::prelude::*;

use crate::{
    components::shell::Shell,
    state::{connection::ConnectionStatus, AppState},
    ws::ws_client,
};

pub mod components;
pub mod nav;
pub mod state;
pub mod views;
pub mod ws;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let state = use_context_provider(AppState::new);
    let status = use_context_provider(|| Signal::new(ConnectionStatus::Connecting));

    let client = ws_client("ws://127.0.0.1:8081".into(), state, status);
    use_context_provider(|| client);

    rsx! {
        Shell { status }
    }
}