use super::operator::OPERATORS;
use crate::search::{ENTRY, NEW_ENTRY, PAGE};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchField {
    Any,
    Name,
    Kind,
    Id,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Query {
    pub scope: u8,
    pub field: SearchField,
    pub fields: Vec<SearchField>,
    pub kinds: Vec<String>,
    pub extensions: Vec<String>,
    pub chapters: Vec<String>,
    pub fuzzy: String,
    pub impossible: bool,
    pub incomplete: bool,
}

impl Query {
    pub fn parse(input: &str, default: u8) -> Self {
        let mut scope = None;
        let mut field = SearchField::Any;
        let mut fields = Vec::new();
        let mut kinds = Vec::new();
        let mut extensions = Vec::new();
        let mut chapters = Vec::new();
        let mut fuzzy = Vec::new();
        let mut incomplete = false;

        for token in lex(input) {
            let Some(operator) = token.strip_prefix('!') else {
                fuzzy.push(token);
                continue;
            };

            let (name, value) = operator.split_once(':').unwrap_or((operator, ""));
            let name = name.to_ascii_lowercase();
            let has_value = operator.contains(':');
            let known = OPERATORS.iter().any(|definition| {
                definition.name == name
                    || definition.aliases.iter().any(|alias| *alias == name)
            });

            if !known {
                fuzzy.push(token);
                continue;
            }

            match name.as_str() {
                "entry" | "e" => {
                    if has_value {
                        if value.is_empty() {
                            incomplete = true;
                            intersect_scope(&mut scope, ENTRY);
                        } else if value.starts_with('[') {
                            intersect_scope(&mut scope, ENTRY);
                            let selected = parse_fields(value);
                            if selected.is_empty() { incomplete = true; }
                            fields.extend(selected);
                        } else {
                            match value.to_ascii_lowercase().as_str() {
                                "name" => { intersect_scope(&mut scope, ENTRY); field = SearchField::Name; }
                                "kind" | "type" => { intersect_scope(&mut scope, ENTRY); field = SearchField::Kind; }
                                "id" => { intersect_scope(&mut scope, ENTRY); field = SearchField::Id; }
                                _ => fuzzy.push(token),
                            }
                        }
                    } else {
                        intersect_scope(&mut scope, ENTRY);
                    }
                }
                "page" | "p" => {
                    if has_value {
                        if value.is_empty() {
                            incomplete = true;
                            intersect_scope(&mut scope, PAGE);
                        } else if value.starts_with('[') {
                            intersect_scope(&mut scope, PAGE);
                            let selected = parse_fields(value);
                            if selected.is_empty() { incomplete = true; }
                            fields.extend(selected);
                        } else {
                            match value.to_ascii_lowercase().as_str() {
                                "name" => { intersect_scope(&mut scope, PAGE); field = SearchField::Name; }
                                "kind" | "type" => { intersect_scope(&mut scope, PAGE); field = SearchField::Kind; }
                                "id" => { intersect_scope(&mut scope, PAGE); field = SearchField::Id; }
                                _ => fuzzy.push(token),
                            }
                        }
                    } else {
                        intersect_scope(&mut scope, PAGE);
                    }
                }
                "ne" | "ee" => {
                    intersect_scope(&mut scope, if name == "ne" { NEW_ENTRY } else { ENTRY });
                    if has_value {
                        let values = values(value);
                        if values.is_empty() { incomplete = true; }
                        kinds.extend(values);
                    }
                }
                "id" => {
                    if has_value { fuzzy.push(token); } else { field = SearchField::Id; }
                }
                "kind" | "k" | "type" => {
                    if has_value {
                        intersect_scope(&mut scope, PAGE | ENTRY);
                        let values = values(value);
                        if values.is_empty() { incomplete = true; }
                        kinds.extend(values);
                    } else {
                        intersect_scope(&mut scope, ENTRY);
                        field = SearchField::Kind;
                    }
                }
                "extension" | "ext" => {
                    let values = values(value);
                    if values.is_empty() { incomplete = true; }
                    extensions.extend(values);
                }
                "chapter" | "c" => {
                    let values = values(value);
                    if values.is_empty() { incomplete = true; }
                    chapters.extend(values);
                }
                _ => fuzzy.push(token),
            }

        }

        let scope = scope.unwrap_or(default);
        Self {
            scope,
            field,
            fields,
            kinds,
            extensions,
            chapters,
            fuzzy: fuzzy.join(" "),
            impossible: scope == 0,
            incomplete,
        }
    }
}

fn intersect_scope(scope: &mut Option<u8>, next: u8) {
    *scope = Some(scope.map_or(next, |current| current & next));
}

pub fn scope(input: &str, default: u8) -> u8 {
    Query::parse(input, default).scope
}

fn values(value: &str) -> Vec<String> {
    let value = value.trim();
    let value = value.strip_prefix('[').and_then(|v| v.strip_suffix(']')).unwrap_or(value);
    let mut values = Vec::new();
    let mut current = String::new();
    let mut escaped = false;

    for character in value.chars() {
        if escaped {
            current.push(character);
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == ',' {
            let item = current.trim();
            if !item.is_empty() {
                values.push(item.to_owned());
            }
            current.clear();
        } else {
            current.push(character);
        }
    }

    if escaped {
        current.push('\\');
    }

    let item = current.trim();
    if !item.is_empty() {
        values.push(item.to_owned());
    }

    values
}

fn parse_fields(value: &str) -> Vec<SearchField> {
    values(value)
        .into_iter()
        .filter_map(|field| match field.to_ascii_lowercase().as_str() {
            "name" => Some(SearchField::Name),
            "kind" | "type" => Some(SearchField::Kind),
            "id" => Some(SearchField::Id),
            _ => None,
        })
        .collect()
}

fn lex(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut token = String::new();
    let mut quote = None;
    let mut bracket_depth = 0usize;
    let mut escaped = false;

    for character in input.chars() {
        if escaped {
            token.push('\\');
            token.push(character);
            escaped = false;
            continue;
        }

        if character == '\\' {
            escaped = true;
            continue;
        }

        if let Some(current) = quote {
            if character == current {
                quote = None;
            } else {
                token.push(character);
            }
            continue;
        }

        match character {
            '"' | '\'' => quote = Some(character),
            '[' => {
                bracket_depth += 1;
                token.push(character);
            }
            ']' => {
                bracket_depth = bracket_depth.saturating_sub(1);
                token.push(character);
            }
            value if value.is_whitespace() && bracket_depth == 0 => {
                if !token.is_empty() {
                    tokens.push(std::mem::take(&mut token));
                }
            }
            value => token.push(value),
        }
    }

    if escaped {
        token.push('\\');
    }
    if !token.is_empty() {
        tokens.push(token);
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aliases_and_scopes() {
        assert_eq!(Query::parse("!e", PAGE | ENTRY).scope, ENTRY);
        assert_eq!(Query::parse("!p", PAGE | ENTRY).scope, PAGE);
        assert_eq!(Query::parse("!ne", PAGE | ENTRY).scope, NEW_ENTRY);
        assert_eq!(Query::parse("!ee", PAGE | ENTRY).scope, ENTRY);
    }

    #[test]
    fn conflicting_scopes_match_nothing() {
        let query = Query::parse("!page !entry", PAGE | ENTRY);
        assert_eq!(query.scope, 0);
        assert!(query.impossible);
    }

    #[test]
    fn filters_combine_and_values_are_or() {
        let query = Query::parse("!kind:[basic:message,trigger:command] !chapter:\"Act One\"", PAGE | ENTRY);
        assert_eq!(query.kinds, vec![String::from("basic:message"), String::from("trigger:command")]);
        assert_eq!(query.chapters, vec![String::from("Act One")]);
    }


    #[test]
    fn page_and_entry_can_select_multiple_fields() {
        let page = Query::parse("!page:[name, type, id]", PAGE | ENTRY);
        assert_eq!(page.scope, PAGE);
        assert_eq!(page.fields, vec![SearchField::Name, SearchField::Kind, SearchField::Id]);

        let entry = Query::parse("!entry:[name,id]", PAGE | ENTRY);
        assert_eq!(entry.scope, ENTRY);
        assert_eq!(entry.fields, vec![SearchField::Name, SearchField::Id]);
    }

    #[test]
    fn keeps_unknown_operators_as_search_terms() {
        let query = Query::parse("!unknown something", PAGE | ENTRY);
        assert_eq!(query.fuzzy, "!unknown something");
    }


    #[test]
    fn kind_filter_keeps_pages_and_existing_entries_only() {
        let query = Query::parse("!kind:basic:message", PAGE | ENTRY | NEW_ENTRY);
        assert_eq!(query.scope, PAGE | ENTRY);
    }

    #[test]
    fn supports_field_search_and_escaped_spaces() {
        let query = Query::parse("!id my\\ page", PAGE | ENTRY);
        assert_eq!(query.field, SearchField::Id);
        assert_eq!(query.fuzzy, "my\\ page");
    }

    #[test]
    fn escaped_filter_values_keep_spaces_and_commas() {
        let query = Query::parse(r"!chapter:Act\ One !ext:[basic,custom\,pack]", PAGE | ENTRY);
        assert_eq!(query.chapters, vec![String::from("Act One")]);
        assert_eq!(query.extensions, vec![String::from("basic"), String::from("custom,pack")]);
    }
}
