use dioxus::prelude::*;

use crate::{
    components::Panel,
    state::{app::AppState, ui::UiState},
    ws::ws_client,
};

pub mod actions;
pub mod components;
pub mod search;
pub mod state;
pub mod ws;

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    use_context_provider(AppState::default);
    use_context_provider(UiState::default);

    let client = ws_client();

    use_context_provider(|| client);

    rsx! {
        Panel {}
    }
}
