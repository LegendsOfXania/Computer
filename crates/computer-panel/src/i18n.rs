use std::collections::HashMap;

use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    French,
}

impl Language {
    fn from_browser(language: &str) -> Self {
        if language.starts_with("fr") {
            Self::French
        } else {
            Self::English
        }
    }

    fn asset(self) -> &'static str {
        match self {
            Self::English => include_str!("../assets/locales/en.toml"),
            Self::French => include_str!("../assets/locales/fr.toml"),
        }
    }
}

#[derive(Clone)]
pub struct I18n {
    translations: Signal<HashMap<String, String>>,
}

impl I18n {
    pub fn t(&self, key: &str) -> String {
        self.translations
            .read()
            .get(key)
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }
}

pub fn use_i18n() -> I18n {
    use_context::<I18n>()
}

pub fn use_i18n_root() {
    let translations = use_signal(HashMap::new);

    use_context_provider(|| I18n { translations });

    use_effect(move || {
        spawn(async move {
            let language = document::eval("return navigator.language")
                .await
                .ok()
                .and_then(|value| value.as_str().map(String::from))
                .map(|language| Language::from_browser(&language))
                .unwrap_or(Language::English);

            let loaded = toml::from_str::<HashMap<String, String>>(language.asset())
                .unwrap_or_default();

            let mut translations = translations;
            translations.set(loaded);
        });
    });
}