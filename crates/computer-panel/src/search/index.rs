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
        let mut matcher = Matcher::from_patterns(&patterns, &Config::default());

        if let [pattern] = patterns.as_slice() {
            let needle = pattern.needle.to_lowercase();
            if !pattern.negated && !needle.is_empty() {
                for (rank, document) in ranks.iter_mut().zip(&candidates) {
                    *rank = exact_tier(document, &needle, fields).map(|tier| (tier, u16::MAX));
                }
            }
        }

        if fields & NAME != 0 {
            fuzzy(&mut matcher, &mut ranks, &candidates, 4, |document| {
                &document.name
            });
        }

        if query.fields == 0 {
            fuzzy(&mut matcher, &mut ranks, &candidates, 5, |document| {
                &document.metadata
            });
        } else {
            if fields & ID != 0 {
                fuzzy(&mut matcher, &mut ranks, &candidates, 5, |document| {
                    &document.id
                });
            }
            if fields & KIND != 0 {
                fuzzy(&mut matcher, &mut ranks, &candidates, 5, |document| {
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
    matcher: &mut Matcher,
    ranks: &mut [Option<(u8, u16)>],
    candidates: &[&'a Document<T>],
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
    for matched in matcher.match_list(&haystacks) {
        ranks[open[matched.index as usize]] = Some((tier, matched.score));
    }
}