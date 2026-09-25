use std::collections::HashMap;

use dioxus::prelude::*;

const DEFAULT_LOCALE: &str = "en";

struct Locale {
    code: &'static str,
    translations: &'static str,
}

const LOCALES: &[Locale] = &[
    Locale {
        code: "en",
        translations: include_str!("../assets/locale/en.toml"),
    },
    Locale {
        code: "fr",
        translations: include_str!("../assets/locale/fr.toml"),
    },
];

impl Locale {
    fn from_browser(language: &str) -> &'static Self {
        let code = language.split('-').next().unwrap_or(language);
        
        if let Some(locale) = LOCALES.iter().find(|locale| locale.code == code) {
            return locale;
        }
        
        if let Some(locale) = LOCALES
            .iter()
            .find(|locale| locale.code == DEFAULT_LOCALE)
        {
            return locale;
        }
            
        &LOCALES[0]
    }

    fn load(&self) -> HashMap<String, String> {
        toml::from_str(self.translations).unwrap_or_default()
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
            .unwrap_or_else(|| key.to_owned())
    }
}

pub fn use_i18n() -> I18n {
    use_context()
}

pub fn use_i18n_root() {
    let mut translations = use_signal(HashMap::new);

    use_context_provider(|| I18n { translations });

    use_effect(move || {
        spawn(async move {
            let language = document::eval("navigator.language")
                .await
                .ok()
                .and_then(|value| value.as_str().map(str::to_owned))
                .unwrap_or_default();

            translations.set(Locale::from_browser(&language).load());
        });
    });
}