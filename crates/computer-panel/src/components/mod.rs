mod editor;
mod header;
mod inspector;
mod search;
mod utils;

use dioxus::prelude::*;
use dioxus_i18n::{prelude::*, unic_langid::langid};

use crate::{
    components::{editor::Editor, header::Header, search::Search, utils::toast::ToastProvider},
    state::ui::UiState,
};

#[component]
pub fn Panel() -> Element {
    let ui = use_context::<UiState>();

    use_init_i18n(|| {
        let lang = js_sys::Reflect::get(&js_sys::global(), &"navigator".into())
            .and_then(|n| js_sys::Reflect::get(&n, &"language".into()))
            .ok()
            .and_then(|l| l.as_string())
            .unwrap_or_default();

        let initial = match lang.split('-').next() {
            Some("fr") => langid!("fr-FR"),
            _ => langid!("en-US"),
        };

        I18nConfig::new(initial)
            .with_fallback(langid!("en-US"))
            .with_locale(Locale::new_static(
                langid!("en-US"),
                include_str!("../../assets/locale/en-US.ftl"),
            ))
            .with_locale(Locale::new_static(
                langid!("fr-FR"),
                include_str!("../../assets/locale/fr-FR.ftl"),
            ))
    });

    rsx! {
        Stylesheet { href: asset!("/assets/style/mod.css") }
        Stylesheet { href: asset!("/assets/style/utils/icon.css") }
        Stylesheet { href: asset!("/assets/style/utils/btn.css") }
        Stylesheet { href: asset!("/assets/style/utils/toast.css") }

        ToastProvider {
            Header {}
            Editor {}

            if let Some((scope, pick)) = ui.get_search() {
                Search { scope, pick }
            }
        }
    }
}
