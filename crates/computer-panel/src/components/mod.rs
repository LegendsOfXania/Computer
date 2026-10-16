mod btn;
mod editor;
mod header;
mod icon;
mod toast;

use dioxus::prelude::*;
use dioxus_i18n::{prelude::*, unic_langid::langid};

use crate::components::{editor::Editor, header::Header, toast::ToastProvider};

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
        document::Stylesheet { href: asset!("/assets/style/icon.css") }
        document::Stylesheet { href: asset!("/assets/style/btn.css") }
        document::Stylesheet { href: asset!("/assets/style/toast.css") }

        ToastProvider {
            Header {}
            Editor {}
        }
    }
}