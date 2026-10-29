use dioxus::prelude::*;

use crate::{
    components::Panel,
    state::{ConnectionStatus, app::AppState, ui::UiState},
    ws::ws_client,
};

pub mod components;
pub mod state;
pub mod ws;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    use_context_provider(|| Signal::new(ConnectionStatus::Connecting));

    use_context_provider(|| {
        let mut state = AppState::new();
        state::mock::load(&mut state);
        state
    });
    use_context_provider(UiState::new);

    let client = ws_client();

    use_context_provider(|| client);

    rsx! {
        Panel {}
    }
}
