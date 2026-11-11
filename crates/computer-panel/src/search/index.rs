use std::cmp::Reverse;

use frizbee::{Config, Matcher, Pattern};

use super::query::{FIELDS, ID, KIND, NAME, Query};

#[derive(Default)]
pub struct Fields<'a> {
    pub name: &'a str,
    pub kind: &'a str,
    pub id: &'a str,
    pub chapter: &'a str,
    pub parent: &'a str,
    pub tags: &'a [String],
}

#[derive(PartialEq)]
struct Document<T> {
    target: T,
    scope: u8,
    name: String,
    kind: String,
    id: String,
    extension: String,
    chapter: String,
    metadata: String,
    keys: [String; 3],
}

#[derive(PartialEq)]
pub struct SearchIndex<T>(Vec<Document<T>>);

impl<T> Default for SearchIndex<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SearchIndex<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, scope: u8, target: T, fields: Fields) {
        let Fields {
            name,
            kind,
            id,
            chapter,
            parent,
            tags,
        } = fields;
        let extension = extension(kind);
        let metadata = [kind, id, extension, chapter, parent, &tags.join(" ")].join(" ");

        self.0.push(Document {
            target,
            scope,
            name: name.into(),
            kind: kind.into(),
            id: id.into(),
            extension: extension.into(),
            chapter: chapter.into(),
            metadata,
            keys: [name, id, kind].map(str::to_lowercase),
        });
    }

    pub fn search(&self, input: &str, default: u8) -> Vec<&T> {
        let query = Query::parse(input, default);
        if query.scope == 0 || query.incomplete {
            return Vec::new();
        }

        let candidates: Vec<&Document<T>> = self
            .0
            .iter()
            .filter(|document| {
                document.scope & query.scope != 0
                    && allows(&query.kinds, &document.kind)
                    && allows(&query.extensions, &document.extension)
                    && allows(&query.chapters, &document.chapter)
            })
            .collect();

        if query.fuzzy.is_empty() {
            return candidates
                .into_iter()
                .map(|document| &document.target)
                .collect();
        }

        let patterns: Vec<Pattern> = Pattern::parse_query(&query.fuzzy)
            .into_iter()
            .map(|pattern| {
                let typos = (pattern.needle.chars().count() / 4).min(2) as u16;
                pattern.max_typos(Some(typos))
            })
            .collect();
        if patterns.is_empty() {
            return Vec::new();
        }

        let fields = if query.fields == 0 {
            FIELDS
        } else {
            query.fields
        };
        let mut ranks: Vec<Option<(u8, u16)>> = vec![None; candidates.len()];

        if let [pattern] = patterns.as_slice() {
            let needle = pattern.needle.to_lowercase();
            if !pattern.negated && !needle.is_empty() {
                for (rank, document) in ranks.iter_mut().zip(&candidates) {
                    *rank = exact_tier(document, &needle, fields).map(|tier| (tier, u16::MAX));
                }
            }
        }

        if fields & NAME != 0 {
            fuzzy(&mut ranks, &candidates, &patterns, 4, |document| {
                &document.name
            });
        }

        if query.fields == 0 {
            fuzzy(&mut ranks, &candidates, &patterns, 5, |document| {
                &document.metadata
            });
        } else {
            if fields & ID != 0 {
                fuzzy(&mut ranks, &candidates, &patterns, 5, |document| {
                    &document.id
                });
            }
            if fields & KIND != 0 {
                fuzzy(&mut ranks, &candidates, &patterns, 5, |document| {
                    &document.kind
                });
            }
        }

        let mut hits: Vec<_> = ranks
            .into_iter()
            .enumerate()
            .filter_map(|(index, rank)| rank.map(|(tier, score)| (tier, Reverse(score), index)))
            .collect();
        hits.sort_unstable();

        hits.into_iter()
            .map(|(.., index)| &candidates[index].target)
            .collect()
    }
}

pub fn extension(kind: &str) -> &str {
    kind.split_once(':').map_or("", |(extension, _)| extension)
}

fn allows(filters: &[String], value: &str) -> bool {
    filters.is_empty()
        || filters
            .iter()
            .any(|filter| filter.eq_ignore_ascii_case(value))
}

fn exact_tier<T>(document: &Document<T>, needle: &str, fields: u8) -> Option<u8> {
    let [name, id, kind] = &document.keys;

    if fields & NAME != 0 && name == needle {
        Some(0)
    } else if fields & NAME != 0 && name.starts_with(needle) {
        Some(1)
    } else if fields & ID != 0 && !id.is_empty() && id == needle {
        Some(2)
    } else if fields & KIND != 0 && kind == needle {
        Some(3)
    } else {
        None
    }
}

fn fuzzy<'a, T>(
    ranks: &mut [Option<(u8, u16)>],
    candidates: &[&'a Document<T>],
    patterns: &[Pattern],
    tier: u8,
    field: impl Fn(&'a Document<T>) -> &'a str,
) {
    let open: Vec<usize> = (0..ranks.len())
        .filter(|&index| ranks[index].is_none())
        .collect();
    if open.is_empty() {
        return;
    }

    let haystacks: Vec<&str> = open.iter().map(|&index| field(candidates[index])).collect();
    let mut matcher = Matcher::from_patterns(patterns, &Config::default());

    for matched in matcher.match_list(&haystacks) {
        ranks[open[matched.index as usize]] = Some((tier, matched.score));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::{ENTRY, NEW_ENTRY, PAGE};

    fn index() -> SearchIndex<&'static str> {
        let mut index = SearchIndex::new();
        let fields = Fields {
            name: "Chapter One",
            kind: "sequence",
            id: "page01",
            chapter: "Act One",
            ..Fields::default()
        };
        index.add(PAGE, "page", fields);

        let tags = ["trigger".to_owned()];
        let fields = Fields {
            name: "Player join",
            kind: "trigger:player_join",
            id: "entry01",
            chapter: "Act One",
            parent: "Chapter One",
            tags: &tags,
        };
        index.add(ENTRY, "entry", fields);

        let tags = ["zzzmove".to_owned()];
        let fields = Fields {
            name: "basic:message",
            kind: "basic:message",
            tags: &tags,
            ..Fields::default()
        };
        index.add(NEW_ENTRY, "new", fields);

        let tags = ["static".to_owned()];
        let fields = Fields {
            name: "basic:fact",
            kind: "basic:fact",
            tags: &tags,
            ..Fields::default()
        };
        index.add(NEW_ENTRY, "other", fields);
        index
    }

    fn found<'a>(index: &'a SearchIndex<&'static str>, query: &str, scope: u8) -> Vec<&'a str> {
        index.search(query, scope).into_iter().copied().collect()
    }

    #[test]
    fn extension_is_derived_from_the_kind() {
        assert_eq!(extension("basic:message"), "basic");
        assert_eq!(extension("message"), "");
    }

    #[test]
    fn chapter_filters_include_pages_and_entries() {
        let index = index();
        assert_eq!(
            found(&index, r#"!chapter:"Act One""#, PAGE | ENTRY),
            ["page", "entry"]
        );
        assert!(found(&index, "!chapter:Nowhere", PAGE | ENTRY).is_empty());
    }

    #[test]
    fn extension_and_kind_filters_are_conjunctive() {
        let index = index();
        assert_eq!(
            found(&index, "!ne:basic:message !ext:basic", NEW_ENTRY),
            ["new"]
        );
        assert!(found(&index, "!ne:basic:message !ext:trigger", NEW_ENTRY).is_empty());
    }

    #[test]
    fn filters_alone_list_everything_that_matches() {
        let index = index();
        assert_eq!(found(&index, "!ne", PAGE | ENTRY), ["new", "other"]);
        assert_eq!(found(&index, "", PAGE | ENTRY), ["page", "entry"]);
        assert_eq!(found(&index, "", PAGE), ["page"]);
    }

    #[test]
    fn incomplete_and_conflicting_operators_match_nothing() {
        let index = index();
        assert!(found(&index, "!kind:", PAGE | ENTRY).is_empty());
        assert!(found(&index, "!page !entry", PAGE | ENTRY).is_empty());
    }

    #[test]
    fn escaped_and_quoted_spaces_are_single_terms() {
        let index = index();
        assert_eq!(found(&index, r"player\ join", ENTRY), ["entry"]);
        assert_eq!(found(&index, r#""player join""#, ENTRY), ["entry"]);
    }

    #[test]
    fn apostrophes_do_not_swallow_the_query() {
        let mut index = SearchIndex::new();
        let fields = Fields {
            name: "Player's join",
            kind: "basic:fact",
            id: "e1",
            ..Fields::default()
        };
        index.add(ENTRY, "entry", fields);
        assert_eq!(found(&index, "player's join", ENTRY), ["entry"]);
        assert_eq!(found(&index, "'join", ENTRY), ["entry"]);
    }

    #[test]
    fn field_selectors_search_only_the_requested_fields() {
        let index = index();
        assert_eq!(found(&index, "!page:kind sequence", PAGE | ENTRY), ["page"]);
        assert!(found(&index, "!page:name sequence", PAGE | ENTRY).is_empty());
        assert_eq!(
            found(&index, "!entry:kind player_join", PAGE | ENTRY),
            ["entry"]
        );
        assert!(found(&index, "!entry:name trigger:player_join", PAGE | ENTRY).is_empty());
        assert_eq!(found(&index, "!id entry01", PAGE | ENTRY), ["entry"]);
        assert_eq!(
            found(&index, "!entry:[name,id] entry01", PAGE | ENTRY),
            ["entry"]
        );
    }

    #[test]
    fn unqualified_search_reads_the_metadata_but_not_new_entries() {
        let index = index();
        assert_eq!(found(&index, "trigger", PAGE | ENTRY), ["entry"]);
        assert!(found(&index, "zzzmove", PAGE | ENTRY).is_empty());
        assert_eq!(found(&index, "zzzmove", NEW_ENTRY), ["new"]);
        assert_eq!(found(&index, "chapter\\ one", ENTRY), ["entry"]);
    }

    #[test]
    fn exact_names_rank_before_prefix_and_fuzzy_matches() {
        let mut index = SearchIndex::new();
        for (target, name) in [
            ("prefix", "Welcome home"),
            ("exact", "Welcome"),
            ("fuzzy", "Welcomer"),
            ("far", "A welcome"),
        ] {
            let fields = Fields {
                name,
                kind: "sequence",
                ..Fields::default()
            };
            index.add(PAGE, target, fields);
        }
        let targets = found(&index, "welcome", PAGE);
        assert_eq!(targets[..2], ["exact", "prefix"]);
        assert_eq!(targets.len(), 4);
    }

    #[test]
    fn names_rank_before_metadata_matches() {
        let mut index = SearchIndex::new();
        let by_chapter = Fields {
            name: "Zzz",
            kind: "sequence",
            chapter: "story",
            ..Fields::default()
        };
        let by_name = Fields {
            name: "Story time",
            kind: "sequence",
            ..Fields::default()
        };
        index.add(PAGE, "by-chapter", by_chapter);
        index.add(PAGE, "by-name", by_name);
        assert_eq!(found(&index, "story", PAGE), ["by-name", "by-chapter"]);
    }

    #[test]
    fn negated_atoms_exclude_matches() {
        let index = index();
        assert_eq!(found(&index, "!join", ENTRY), Vec::<&str>::new());
    }

    #[test]
    fn operators_without_terms_never_panic() {
        let index = index();
        for query in [
            "!",
            "^",
            "'",
            "$",
            "!!",
            "\\",
            "! !",
            "\"",
            "[",
            "!kind:[",
            "é\u{a0}!",
        ] {
            let _ = index.search(query, PAGE | ENTRY);
        }
    }
}
