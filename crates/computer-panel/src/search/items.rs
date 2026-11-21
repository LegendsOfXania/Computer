use std::collections::HashMap;

use computer_model::{
    EntryDefinition, Library,
    id::{EntryId, PageId},
    page::PageKind,
};

use super::{ENTRY, Fields, NEW_ENTRY, PAGE, SearchIndex, extension, scope};

const LIMIT: usize = 50;

pub type Sections = [Vec<SearchItem>; 3];

#[derive(Clone, PartialEq)]
pub enum Target {
    Page(PageId),
    Entry(EntryId),
    NewEntry(String),
    NewPage,
}

impl Target {
    fn section(&self) -> usize {
        match self {
            Self::Entry(_) => 0,
            Self::NewEntry(_) => 1,
            Self::Page(_) | Self::NewPage => 2,
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct SearchItem {
    pub target: Target,
    pub icon: String,
    pub color: String,
    pub label: String,
    pub hint: String,
    pub meta: String,
}

impl SearchItem {
    fn new(target: Target, icon: &str, color: &str, label: &str, hint: &str, meta: &str) -> Self {
        Self {
            target,
            icon: icon.into(),
            color: color.into(),
            label: label.into(),
            hint: hint.into(),
            meta: meta.into(),
        }
    }
}

#[must_use]
pub fn build_index(
    library: &Library,
    registry: &HashMap<String, EntryDefinition>,
    new_entry: &str,
) -> SearchIndex<SearchItem> {
    let mut index = SearchIndex::new();

    let mut pages: Vec<_> = library.pages.values().collect();

    pages.sort_by(|a, b| (a.priority, &a.name).cmp(&(b.priority, &b.name)));

    for page in pages {
        let chapter = page.chapter.as_deref().unwrap_or("");
        let kind = page.kind.as_str();
        let id = page.id.to_string();

        index.add(
            PAGE,
            SearchItem::new(
                Target::Page(page.id),
                page.kind.icon(),
                "",
                &page.name,
                kind,
                chapter,
            ),
            Fields {
                name: &page.name,
                kind,
                id: &id,
                chapter,
                ..Fields::default()
            },
        );

        let place = if chapter.is_empty() {
            page.name.clone()
        } else {
            format!("{chapter} · {}", page.name)
        };

        for entry in page.entries.iter().filter_map(|id| library.entries.get(id)) {
            let definition = registry.get(&entry.kind);
            let (icon, color) =
                definition.map_or(("", ""), |d| (d.icon.as_str(), d.color.as_str()));
            let id = entry.id.to_string();

            index.add(
                ENTRY,
                SearchItem::new(
                    Target::Entry(entry.id()),
                    icon,
                    color,
                    entry.label(),
                    &entry.kind,
                    &place,
                ),
                Fields {
                    name: entry.label(),
                    kind: &entry.kind,
                    id: &id,
                    chapter,
                    parent: &page.name,
                    tags: definition.map_or(&[], |d| d.tags.as_slice()),
                },
            );
        }
    }

    let mut definitions: Vec<_> = registry.values().collect();

    definitions.sort_by(|a, b| a.kind.cmp(&b.kind));

    for definition in definitions {
        let kind = &definition.kind;

        index.add(
            NEW_ENTRY,
            SearchItem::new(
                Target::NewEntry(kind.clone()),
                &definition.icon,
                &definition.color,
                kind,
                new_entry,
                extension(kind),
            ),
            Fields {
                name: kind,
                kind,
                tags: &definition.tags,
                ..Fields::default()
            },
        );
    }

    index
}

#[must_use]
pub fn build_sections(
    library: &Library,
    index: &SearchIndex<SearchItem>,
    query: &str,
    wanted: u8,
    accepts: Option<&EntryDefinition>,
    new_page: &str,
) -> Sections {
    let default = if accepts.is_some() { PAGE } else { wanted };
    let mut sections = Sections::default();

    for item in index.search(query, default) {
        if let Some(definition) = accepts {
            let Target::Page(id) = &item.target else {
                continue;
            };

            if !library
                .pages
                .get(id)
                .is_some_and(|page| page.kind.accepts(definition))
            {
                continue;
            }
        }

        if let Some(items) = sections.get_mut(item.target.section())
            && items.len() < LIMIT
        {
            items.push(item.clone());

            if sections.iter().all(|items| items.len() == LIMIT) {
                break;
            }
        }
    }

    let can_create_page =
        accepts.is_none_or(|definition| PageKind::ALL.iter().any(|kind| kind.accepts(definition)));

    if can_create_page && scope(query, default) & PAGE != 0 {
        let [_, _, pages] = &mut sections;

        pages.push(SearchItem::new(
            Target::NewPage,
            "plus",
            "",
            new_page,
            "",
            "",
        ));
    }

    sections
}
