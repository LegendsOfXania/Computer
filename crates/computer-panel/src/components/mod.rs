mod editor;
mod header;
mod inspector;
mod utils;

use dioxus::prelude::*;
use dioxus_i18n::{prelude::*, unic_langid::langid};

use crate::components::{editor::Editor, header::Header, utils::toast::ToastProvider};

#[component]
pub fn Panel() -> Element {
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
        document::Stylesheet { href: asset!("/assets/style/mod.css") }
        document::Stylesheet { href: asset!("/assets/style/utils/icon.css") }
        document::Stylesheet { href: asset!("/assets/style/utils/btn.css") }
        document::Stylesheet { href: asset!("/assets/style/utils/toast.css") }
        document::Stylesheet { href: asset!("/assets/style/inspector/window.css") }
        document::Stylesheet { href: asset!("/assets/style/inspector/fields.css") }

        ToastProvider {
            Header {}
            Editor {}
        }
    }
}
