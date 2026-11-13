use super::{ENTRY, NEW_ENTRY, PAGE};

pub(super) const NAME: u8 = 1;
pub(super) const ID: u8 = 2;
pub(super) const KIND: u8 = 4;
pub(super) const FIELDS: u8 = NAME | ID | KIND;

const OPERATORS: [(&str, &[&str], bool); 8] = [
    ("entry", &["e"], false),
    ("page", &["p"], false),
    ("ne", &[], false),
    ("ee", &[], false),
    ("id", &[], false),
    ("kind", &["k", "type"], true),
    ("extension", &["ext"], true),
    ("chapter", &["c"], true),
];

#[derive(Default)]
pub(super) struct Query {
    pub scope: u8,
    pub fields: u8,
    pub kinds: Vec<String>,
    pub extensions: Vec<String>,
    pub chapters: Vec<String>,
    pub fuzzy: String,
    pub incomplete: bool,
}

impl Query {
    pub fn parse(input: &str, default: u8) -> Self {
        let mut query = Self::new();
        let mut fuzzy = Vec::new();

        for (_, token) in split(input) {
            if !query.apply(token) {
                fuzzy.push(atom(token));
            }
        }

        if query.scope == !0 {
            query.scope = default;
        }
        query.fuzzy = fuzzy.join(" ");
        query
    }

    fn new() -> Self {
        Self {
            scope: !0,
            ..Self::default()
        }
    }

    fn apply(&mut self, token: &str) -> bool {
        let Some(rest) = token.strip_prefix('!') else {
            return false;
        };
        let (name, value) = match rest.split_once(':') {
            Some((name, value)) => (name, Some(value)),
            None => (rest, None),
        };
        let Some(name) = canonical(name) else {
            return false;
        };

        match (name, value) {
            ("id", None) => self.fields |= ID,
            ("kind", None) => {
                self.scope &= ENTRY;
                self.fields |= KIND;
            }
            ("kind", Some(value)) => {
                self.scope &= PAGE | ENTRY;
                collect(&mut self.kinds, &mut self.incomplete, value);
            }
            ("entry" | "page", value) => {
                let Some(fields) = value.map_or(Some(0), field_bits) else {
                    return false;
                };
                self.scope &= if name == "entry" { ENTRY } else { PAGE };
                self.fields |= fields;
                self.incomplete |= value.is_some() && fields == 0;
            }
            ("ne" | "ee", value) => {
                self.scope &= if name == "ne" { NEW_ENTRY } else { ENTRY };
                if let Some(value) = value {
                    collect(&mut self.kinds, &mut self.incomplete, value);
                }
            }
            ("extension", value) => {
                collect(
                    &mut self.extensions,
                    &mut self.incomplete,
                    value.unwrap_or(""),
                );
            }
            ("chapter", value) => {
                collect(
                    &mut self.chapters,
                    &mut self.incomplete,
                    value.unwrap_or(""),
                );
            }
            _ => return false,
        }

        true
    }
}

pub fn scope(input: &str, default: u8) -> u8 {
    Query::parse(input, default).scope
}

pub fn highlight(input: &str) -> Vec<(&str, bool)> {
    let mut parts = Vec::new();
    let mut plain = 0;

    for (start, token) in split(input) {
        if !Query::new().apply(token) {
            continue;
        }

        let end = start + token.find(':').map_or(token.len(), |colon| colon + 1);
        parts.push((&input[plain..start], false));
        parts.push((&input[start..end], true));
        plain = end;
    }

    parts.push((&input[plain..], false));
    parts.retain(|(text, _)| !text.is_empty());
    parts
}

fn canonical(name: &str) -> Option<&'static str> {
    OPERATORS
        .iter()
        .find(|(canonical, aliases, _)| {
            canonical.eq_ignore_ascii_case(name)
                || aliases.iter().any(|alias| alias.eq_ignore_ascii_case(name))
        })
        .map(|(canonical, ..)| *canonical)
}

fn collect(list: &mut Vec<String>, incomplete: &mut bool, value: &str) {
    let found = values(value);
    *incomplete |= found.is_empty();
    list.extend(found);
}

fn field_bit(name: &str) -> u8 {
    match name.to_ascii_lowercase().as_str() {
        "name" => NAME,
        "kind" | "type" => KIND,
        "id" => ID,
        _ => 0,
    }
}

fn field_bits(value: &str) -> Option<u8> {
    let bits = values(value)
        .iter()
        .fold(0, |bits, name| bits | field_bit(name));
    (bits != 0 || value.is_empty() || value.starts_with('[')).then_some(bits)
}

fn split(input: &str) -> Vec<(usize, &str)> {
    let mut tokens = Vec::new();
    let mut start = None;
    let (mut quoted, mut escaped, mut depth) = (false, false, 0usize);

    for (index, character) in input.char_indices() {
        if character.is_whitespace() && !escaped && !quoted && depth == 0 {
            if let Some(from) = start.take() {
                tokens.push((from, &input[from..index]));
            }
            continue;
        }

        start.get_or_insert(index);

        if escaped {
            escaped = false;
            continue;
        }

        match character {
            '\\' => escaped = true,
            '"' => quoted = !quoted,
            '[' if !quoted => depth += 1,
            ']' if !quoted => depth = depth.saturating_sub(1),
            _ => {}
        }
    }

    if let Some(from) = start {
        tokens.push((from, &input[from..]));
    }

    tokens
}

fn atom(token: &str) -> String {
    let mut atom = String::with_capacity(token.len());
    let mut characters = token.chars();

    while let Some(character) = characters.next() {
        match character {
            '\\' => {
                atom.push(character);
                atom.extend(characters.next());
            }
            '"' => {}
            _ if character.is_whitespace() => {
                atom.push('\\');
                atom.push(character);
            }
            _ => atom.push(character),
        }
    }

    atom
}

fn values(value: &str) -> Vec<String> {
    let value = value.strip_prefix('[').unwrap_or(value);
    let value = value.strip_suffix(']').unwrap_or(value);
    let mut values = Vec::new();
    let mut current = String::new();
    let (mut quoted, mut escaped) = (false, false);

    for character in value.chars() {
        if escaped {
            current.push(character);
            escaped = false;
            continue;
        }

        match character {
            '\\' => escaped = true,
            '"' => quoted = !quoted,
            ',' if !quoted => values.push(std::mem::take(&mut current)),
            _ => current.push(character),
        }
    }

    if escaped {
        current.push('\\');
    }
    values.push(current);

    values
        .into_iter()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .collect()
}