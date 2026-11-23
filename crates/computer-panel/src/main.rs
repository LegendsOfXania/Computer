use dioxus::prelude::*;

use crate::{
    components::Panel,
    components::utils::toast::Toasts,
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
    use_context_provider(Toasts::default);

    let client = ws_client();
    use_context_provider(|| client);

    rsx! {
        Panel {}
    }
}
