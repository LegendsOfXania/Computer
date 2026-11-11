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

pub fn complete(input: &str) -> Option<String> {
    let start = input
        .char_indices()
        .rev()
        .find(|(_, character)| character.is_whitespace())
        .map_or(0, |(index, character)| index + character.len_utf8());
    let token = &input[start..];
    let partial = token.strip_prefix('!')?.to_ascii_lowercase();

    if partial.is_empty() || partial.contains(':') {
        return None;
    }

    let mut matches = OPERATORS.iter().filter(|(name, aliases, _)| {
        name.starts_with(&partial) || aliases.iter().any(|alias| alias.starts_with(&partial))
    });
    let (name, _, colon) = matches.next()?;

    if matches.next().is_some() {
        return None;
    }

    let completion = if *colon {
        format!("!{name}:")
    } else {
        format!("!{name}")
    };

    (!token.eq_ignore_ascii_case(&completion)).then(|| format!("{}{completion}", &input[..start]))
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

/// Splits on whitespace that is neither escaped, quoted nor inside brackets.
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

/// A fuzzy token as Frizbee expects it: quotes removed, inner spaces escaped.
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

/// `[a, b]` or `a,b`: comma separated, `\x` escapes, `"..."` keeps commas and spaces.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> Query {
        Query::parse(input, PAGE | ENTRY)
    }

    #[test]
    fn aliases_and_scopes() {
        assert_eq!(parse("!e").scope, ENTRY);
        assert_eq!(parse("!p").scope, PAGE);
        assert_eq!(parse("!ne").scope, NEW_ENTRY);
        assert_eq!(parse("!ee").scope, ENTRY);
        assert_eq!(parse("welcome").scope, PAGE | ENTRY);
    }

    #[test]
    fn conflicting_scopes_match_nothing() {
        assert_eq!(parse("!page !entry").scope, 0);
        assert_eq!(parse("!ne:basic:message !ee").scope, 0);
    }

    #[test]
    fn filters_combine_and_values_are_or() {
        let query = parse(r#"!kind:[basic:message,trigger:command] !chapter:"Act One""#);
        assert_eq!(query.kinds, ["basic:message", "trigger:command"]);
        assert_eq!(query.chapters, ["Act One"]);
        assert_eq!(query.scope, PAGE | ENTRY);
    }

    #[test]
    fn kind_with_a_value_never_selects_new_entries() {
        let query = Query::parse("!kind:basic:message", PAGE | ENTRY | NEW_ENTRY);
        assert_eq!(query.scope, PAGE | ENTRY);
    }

    #[test]
    fn fields_are_selected_by_name_or_list() {
        assert_eq!(parse("!page:name").fields, NAME);
        assert_eq!(parse("!entry:type").fields, KIND);
        assert_eq!(parse("!page:[name, type, id]").fields, FIELDS);
        assert_eq!(parse("!entry:[name,id]").scope, ENTRY);
        assert_eq!(parse("!id").fields, ID);
        assert_eq!(parse("!kind").fields, KIND);
        assert_eq!(parse("!page:name !id").fields, NAME | ID);
    }

    #[test]
    fn unknown_operators_are_search_terms() {
        assert_eq!(parse("!unknown something").fuzzy, "!unknown something");
        assert_eq!(parse("!page:chapter test").fuzzy, "!page:chapter test");
        assert_eq!(parse("!id:message").fuzzy, "!id:message");
        assert_eq!(parse("!page:chapter").scope, PAGE | ENTRY);
    }

    #[test]
    fn empty_values_are_incomplete() {
        for input in [
            "!kind:",
            "!ne:",
            "!ext",
            "!chapter:",
            "!page:",
            "!entry:[]",
            "!page:[foo]",
        ] {
            assert!(parse(input).incomplete, "{input}");
        }
        for input in ["!kind", "!ne", "!ee", "!page", "!id", "!kind:a"] {
            assert!(!parse(input).incomplete, "{input}");
        }
    }

    #[test]
    fn escaped_spaces_and_quotes_are_single_fuzzy_atoms() {
        assert_eq!(parse(r"!id my\ page").fuzzy, r"my\ page");
        assert_eq!(parse(r#""player join" now"#).fuzzy, r"player\ join now");
        assert_eq!(parse(r"player\ join").fuzzy, r"player\ join");
    }

    #[test]
    fn apostrophes_stay_in_the_query() {
        assert_eq!(parse("player's join").fuzzy, "player's join");
        assert_eq!(parse("'exact term").fuzzy, "'exact term");
    }

    #[test]
    fn escaped_filter_values_keep_spaces_and_commas() {
        let query = parse(r"!chapter:Act\ One !ext:[basic,custom\,pack]");
        assert_eq!(query.chapters, ["Act One"]);
        assert_eq!(query.extensions, ["basic", "custom,pack"]);
    }

    #[test]
    fn quoted_values_keep_commas() {
        assert_eq!(parse(r#"!chapter:"A, B""#).chapters, ["A, B"]);
    }

    #[test]
    fn brackets_group_spaces() {
        assert_eq!(parse("!kind:[a, b] x").kinds, ["a", "b"]);
        assert_eq!(parse("!kind:[a, b] x").fuzzy, "x");
    }

    #[test]
    fn highlight_colors_only_what_the_parser_accepts() {
        assert_eq!(
            highlight("!entry cac !kind:basic:fact !page:foo !ext"),
            [
                ("!entry", true),
                (" cac ", false),
                ("!kind:", true),
                ("basic:fact !page:foo ", false),
                ("!ext", true),
            ]
        );
    }

    #[test]
    fn highlight_keeps_every_character() {
        for input in ["", "  ", "a  b ", " !k:x\t!id ", r#"!c:"A B" é ✓"#, r"a\ b"] {
            let joined: String = highlight(input).into_iter().map(|(text, _)| text).collect();
            assert_eq!(joined, input);
        }
    }

    #[test]
    fn completes_common_operator_prefixes() {
        assert_eq!(complete("!en").as_deref(), Some("!entry"));
        assert_eq!(complete("!pa").as_deref(), Some("!page"));
        assert_eq!(complete("!ex").as_deref(), Some("!extension:"));
        assert_eq!(complete("!ki").as_deref(), Some("!kind:"));
        assert_eq!(complete("!k").as_deref(), Some("!kind:"));
        assert_eq!(complete("!ch").as_deref(), Some("!chapter:"));
    }

    #[test]
    fn completes_only_the_current_unfinished_operator() {
        assert_eq!(complete("hello !en").as_deref(), Some("hello !entry"));
        assert_eq!(complete("!chapter:Act"), None);
        assert_eq!(complete("!e"), None);
        assert_eq!(complete("!ee"), None);
        assert_eq!(complete("!entry"), None);
        assert_eq!(complete("hello "), None);
    }

    #[test]
    fn completion_survives_multibyte_whitespace() {
        assert_eq!(complete("a\u{a0}!en").as_deref(), Some("a\u{a0}!entry"));
        assert_eq!(complete("é !pa").as_deref(), Some("é !page"));
    }
}
