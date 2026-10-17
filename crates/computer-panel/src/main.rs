use dioxus::prelude::*;

use crate::{
    components::Panel,
    state::{app::AppState, ui::UiState, ConnectionStatus},
    ws::ws_client,
};

pub mod components;
pub mod id;
pub mod state;
pub mod ws;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let app = use_context_provider(AppState::new);
    use_context_provider(|| Signal::new(ConnectionStatus::Connecting));

    let client = ws_client();

    use_context_provider(|| client);
    let ui = use_context_provider(UiState::new);

    use_hook(|| state::mock::seed(app, ui));

    rsx! {
        Panel {}
    }
}