use std::cmp::Reverse;

use super::query::{Query, SearchField, match_indices, patterns};
use super::{ENTRY, NEW_ENTRY, PAGE};

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
}

#[derive(PartialEq)]
pub struct SearchIndex<T> {
    documents: Vec<Document<T>>,
}

pub struct SearchMatch<'a, T> {
    pub target: &'a T,
    pub score: u16,
}

impl<T> Default for SearchIndex<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SearchIndex<T> {
    pub fn new() -> Self {
        Self {
            documents: Vec::new(),
        }
    }

    pub fn add_page(&mut self, target: T, name: &str, kind: &str, id: &str, chapter: Option<&str>) {
        self.add(
            PAGE,
            target,
            name,
            kind,
            id,
            "",
            chapter.unwrap_or(""),
            "",
            "",
        );
    }

    pub fn add_entry(
        &mut self,
        target: T,
        name: &str,
        kind: &str,
        id: &str,
        extension: &str,
        chapter: Option<&str>,
        parent: Option<&str>,
        tags: &[String],
    ) {
        self.add(
            ENTRY,
            target,
            name,
            kind,
            id,
            extension,
            chapter.unwrap_or(""),
            parent.unwrap_or(""),
            &tags.join(" "),
        );
    }

    pub fn add_new_entry(&mut self, target: T, kind: &str, extension: &str, tags: &[String]) {
        self.add(
            NEW_ENTRY,
            target,
            kind,
            kind,
            "",
            extension,
            "",
            "",
            &tags.join(" "),
        );
    }

    fn add(
        &mut self,
        scope: u8,
        target: T,
        name: &str,
        kind: &str,
        id: &str,
        extension: &str,
        chapter: &str,
        parent: &str,
        tags: &str,
    ) {
        let metadata = [kind, id, extension, chapter, parent, tags].join(" ");
        self.documents.push(Document {
            target,
            scope,
            name: name.to_owned(),
            kind: kind.to_owned(),
            id: id.to_owned(),
            extension: extension.to_owned(),
            chapter: chapter.to_owned(),
            metadata,
        });
    }

    pub fn search(&self, input: &str, default_scope: u8) -> Vec<SearchMatch<'_, T>> {
        let query = Query::parse(input, default_scope);
        if query.impossible || query.incomplete {
            return Vec::new();
        }

        let candidates: Vec<usize> = self
            .documents
            .iter()
            .enumerate()
            .filter(|(_, document)| document.scope & query.scope != 0)
            .filter(|(_, document)| matches_any(&query.kinds, &document.kind))
            .filter(|(_, document)| matches_any(&query.extensions, &document.extension))
            .filter(|(_, document)| matches_any(&query.chapters, &document.chapter))
            .map(|(index, _)| index)
            .collect();

        if query.fuzzy.trim().is_empty() {
            return candidates
                .into_iter()
                .map(|index| SearchMatch {
                    target: &self.documents[index].target,
                    score: 0,
                })
                .collect();
        }

        let parsed = patterns(&query.fuzzy);
        if parsed.is_empty() {
            return Vec::new();
        }

        let mut ranked: Vec<(u8, u16, usize)> = Vec::new();
        let mut included = vec![false; self.documents.len()];

        match query.field {
            SearchField::Id => {
                let values: Vec<&str> = self
                    .documents
                    .iter()
                    .map(|document| document.id.as_str())
                    .collect();
                self.rank_field(
                    &candidates,
                    &values,
                    &parsed,
                    &mut ranked,
                    &mut included,
                    4,
                    Some(0),
                    Some(1),
                );
            }
            SearchField::Kind => {
                let values: Vec<&str> = self
                    .documents
                    .iter()
                    .map(|document| document.kind.as_str())
                    .collect();
                self.rank_field(
                    &candidates,
                    &values,
                    &parsed,
                    &mut ranked,
                    &mut included,
                    4,
                    Some(0),
                    Some(1),
                );
            }
            SearchField::Name => {
                let values: Vec<&str> = self
                    .documents
                    .iter()
                    .map(|document| document.name.as_str())
                    .collect();
                self.rank_field(
                    &candidates,
                    &values,
                    &parsed,
                    &mut ranked,
                    &mut included,
                    4,
                    Some(0),
                    Some(1),
                );
            }
            SearchField::Any if !query.fields.is_empty() => {
                self.rank_selected_exact(
                    &candidates,
                    &parsed,
                    &query.fields,
                    &mut ranked,
                    &mut included,
                );

                if query.fields.contains(&SearchField::Name) {
                    let names: Vec<&str> = self
                        .documents
                        .iter()
                        .map(|document| document.name.as_str())
                        .collect();
                    self.rank_field(
                        &candidates,
                        &names,
                        &parsed,
                        &mut ranked,
                        &mut included,
                        4,
                        None,
                        None,
                    );
                }

                let metadata: Vec<String> = self
                    .documents
                    .iter()
                    .map(|document| {
                        let mut fields = Vec::new();
                        if query.fields.contains(&SearchField::Id) {
                            fields.push(document.id.as_str());
                        }
                        if query.fields.contains(&SearchField::Kind) {
                            fields.push(document.kind.as_str());
                        }
                        fields.join(" ")
                    })
                    .collect();
                let metadata: Vec<&str> = metadata.iter().map(String::as_str).collect();
                self.rank_field(
                    &candidates,
                    &metadata,
                    &parsed,
                    &mut ranked,
                    &mut included,
                    5,
                    None,
                    None,
                );
            }
            SearchField::Any => {
                self.rank_exact(&candidates, &parsed, &mut ranked, &mut included);

                let names: Vec<&str> = self
                    .documents
                    .iter()
                    .map(|document| document.name.as_str())
                    .collect();
                self.rank_field(
                    &candidates,
                    &names,
                    &parsed,
                    &mut ranked,
                    &mut included,
                    4,
                    Some(0),
                    Some(1),
                );

                let metadata: Vec<&str> = self
                    .documents
                    .iter()
                    .map(|document| document.metadata.as_str())
                    .collect();
                self.rank_field(
                    &candidates,
                    &metadata,
                    &parsed,
                    &mut ranked,
                    &mut included,
                    5,
                    None,
                    None,
                );
            }
        }

        ranked.sort_by_key(|(rank, score, index)| (*rank, Reverse(*score), *index));
        ranked
            .into_iter()
            .map(|(_, score, index)| SearchMatch {
                target: &self.documents[index].target,
                score,
            })
            .collect()
    }

    fn rank_selected_exact(
        &self,
        candidates: &[usize],
        parsed: &[frizbee::Pattern],
        fields: &[SearchField],
        ranked: &mut Vec<(u8, u16, usize)>,
        included: &mut [bool],
    ) {
        if parsed.len() != 1 || parsed[0].negated {
            return;
        }

        let needle = parsed[0].needle.to_lowercase();
        for index in candidates {
            let document = &self.documents[*index];
            let name = document.name.to_lowercase();
            let id = document.id.to_lowercase();
            let kind = document.kind.to_lowercase();
            let rank = if fields.contains(&SearchField::Name) && name == needle {
                Some(0)
            } else if fields.contains(&SearchField::Name) && name.starts_with(&needle) {
                Some(1)
            } else if fields.contains(&SearchField::Id) && !id.is_empty() && id == needle {
                Some(2)
            } else if fields.contains(&SearchField::Kind) && kind == needle {
                Some(3)
            } else {
                None
            };

            if let Some(rank) = rank {
                included[*index] = true;
                ranked.push((rank, u16::MAX, *index));
            }
        }
    }

    fn rank_exact(
        &self,
        candidates: &[usize],
        parsed: &[frizbee::Pattern],
        ranked: &mut Vec<(u8, u16, usize)>,
        included: &mut [bool],
    ) {
        if parsed.len() != 1 || parsed[0].negated {
            return;
        }

        let needle = parsed[0].needle.to_lowercase();
        if needle.is_empty() {
            return;
        }

        for index in candidates {
            let document = &self.documents[*index];
            let name = document.name.to_lowercase();
            let id = document.id.to_lowercase();
            let kind = document.kind.to_lowercase();

            let rank = if name == needle {
                Some(0)
            } else if name.starts_with(&needle) {
                Some(1)
            } else if !id.is_empty() && id == needle {
                Some(2)
            } else if kind == needle {
                Some(3)
            } else {
                None
            };

            if let Some(rank) = rank {
                included[*index] = true;
                ranked.push((rank, u16::MAX, *index));
            }
        }
    }

    fn rank_field(
        &self,
        candidates: &[usize],
        values: &[&str],
        parsed: &[frizbee::Pattern],
        ranked: &mut Vec<(u8, u16, usize)>,
        included: &mut [bool],
        rank: u8,
        exact_rank: Option<u8>,
        prefix_rank: Option<u8>,
    ) {
        if let Some(needle) =
            (parsed.len() == 1 && !parsed[0].negated).then(|| parsed[0].needle.to_lowercase())
        {
            for index in candidates {
                if included[*index] {
                    continue;
                }
                let value = values[*index].to_lowercase();
                let matched_rank = if !value.is_empty() && value == needle {
                    exact_rank
                } else if !value.is_empty() && value.starts_with(&needle) {
                    prefix_rank
                } else {
                    None
                };

                if let Some(matched_rank) = matched_rank {
                    included[*index] = true;
                    ranked.push((matched_rank, u16::MAX, *index));
                }
            }
        }

        let haystacks: Vec<&str> = candidates.iter().map(|index| values[*index]).collect();
        for (candidate_index, score) in match_indices(parsed, &haystacks) {
            let Some(index) = candidates.get(candidate_index).copied() else {
                continue;
            };
            if !included[index] {
                included[index] = true;
                ranked.push((rank, score, index));
            }
        }
    }
}

fn matches_any(filters: &[String], value: &str) -> bool {
    filters.is_empty()
        || filters
            .iter()
            .any(|filter| filter.eq_ignore_ascii_case(value))
}

pub fn extension_for_kind(kind: &str) -> &str {
    kind.split_once(':').map_or("", |(extension, _)| extension)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn index() -> SearchIndex<&'static str> {
        let mut index = SearchIndex::new();
        index.add_page("page", "Chapter One", "sequence", "page01", Some("Act One"));
        index.add_entry(
            "entry",
            "Player join",
            "trigger:player_join",
            "entry01",
            "trigger",
            Some("Act One"),
            Some("Chapter One"),
            &["trigger".into()],
        );
        index.add_new_entry("new", "basic:message", "basic", &["action".into()]);
        index.add_new_entry("other", "basic:fact", "basic", &["static".into()]);
        index
    }

    #[test]
    fn extension_is_derived_from_kind() {
        assert_eq!(extension_for_kind("basic:message"), "basic");
        assert_eq!(extension_for_kind("message"), "");
    }

    #[test]
    fn chapter_filters_include_pages_and_entries() {
        let index = index();
        let matches = index.search("!chapter:\"Act One\"", PAGE | ENTRY);
        let targets: Vec<_> = matches.into_iter().map(|matched| *matched.target).collect();
        assert_eq!(targets, vec!["page", "entry"]);
    }

    #[test]
    fn extension_and_kind_filters_are_conjunctive() {
        let index = index();
        let matches = index.search("!ne:basic:message !ext:basic", NEW_ENTRY);
        assert_eq!(matches.len(), 1);
        assert_eq!(*matches[0].target, "new");
    }

    #[test]
    fn escaped_spaces_are_single_fuzzy_terms() {
        let index = index();
        let matches = index.search(r"player\ join", ENTRY);
        assert_eq!(matches.len(), 1);
        assert_eq!(*matches[0].target, "entry");
    }

    #[test]
    fn exact_names_rank_before_prefix_and_fuzzy_matches() {
        let mut index = SearchIndex::new();
        index.add_page("prefix", "Welcome home", "sequence", "p1", None);
        index.add_page("exact", "Welcome", "sequence", "p2", None);
        index.add_page("fuzzy", "Welcomer", "sequence", "p3", None);
        let matches = index.search("welcome", PAGE);
        let targets: Vec<_> = matches.into_iter().map(|matched| *matched.target).collect();
        assert_eq!(targets[0], "exact");
        assert_eq!(targets[1], "prefix");
    }
}
