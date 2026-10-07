use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    components::Panel,
    state::{app::AppState, ui::UiState, ConnectionStatus},
    ws::ws_client,
};

pub mod components;
pub mod i18n;
pub mod state;
pub mod ws;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    use_context_provider(AppState::new);
    use_context_provider(|| Signal::new(ConnectionStatus::Connecting));

    let client = Rc::new(ws_client());

    use_context_provider(|| client);
    use_context_provider(UiState::new);

    rsx! {
        Panel {}
    }
}