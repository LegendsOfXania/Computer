use std::{collections::HashMap, rc::Rc};

use dioxus::prelude::*;

type Strings = HashMap<&'static str, &'static str>;

const FALLBACK: &str = "en";

const LOCALES: &[(&str, &str)] = &[
    ("en", include_str!("../assets/locale/en.lang")),
    ("fr", include_str!("../assets/locale/fr.lang")),
];

fn parse(source: &'static str) -> Strings {
    source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| (key.trim(), value.trim()))
        .collect()
}

fn load(code: &str) -> Option<Strings> {
    LOCALES
        .iter()
        .find(|(c, _)| *c == code)
        .map(|(_, source)| parse(source))
}

#[derive(Clone)]
pub struct I18n {
    current: Signal<Strings>,
    fallback: Rc<Strings>,
}

impl I18n {
    pub fn t(&self, key: &str) -> String {
        let current = self.current.read();
        current
            .get(key)
            .or_else(|| self.fallback.get(key))
            .map_or_else(|| key.to_owned(), |s| (*s).to_owned())
    }
}

pub fn use_i18n() -> I18n {
    use_context()
}

pub fn use_i18n_root() {
    let fallback = use_hook(|| Rc::new(load(FALLBACK).unwrap_or_default()));
    let mut current = use_signal(|| (*fallback).clone());

    use_context_provider(|| I18n { current, fallback });

    use_future(move || async move {
        let Ok(value) = document::eval("return navigator.language;").await else {
            return;
        };
        let language = value.as_str().unwrap_or_default();
        let code = language.split('-').next().unwrap_or_default();

        if let Some(strings) = load(code) {
            current.set(strings);
        }
    });
}