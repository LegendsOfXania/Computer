use computer_model::{id::PageId, page::Page};
use std::collections::BTreeMap;

#[derive(PartialEq)]
pub enum Row {
    Chapter {
        name: String,
        path: String,
        depth: usize,
    },
    Page {
        id: PageId,
        name: String,
        icon: &'static str,
        depth: usize,
    },
}

#[derive(Default)]
struct Node {
    pages: Vec<(i32, PageId, String, &'static str)>,
    children: BTreeMap<String, Node>,
}

impl Node {
    fn priority(&self) -> i32 {
        self.pages
            .iter()
            .map(|page| page.0)
            .chain(self.children.values().map(Node::priority))
            .min()
            .unwrap_or(i32::MAX)
    }

    fn flatten(mut self, prefix: &str, depth: usize, rows: &mut Vec<Row>) {
        self.pages.sort_by_key(|page| page.0);

        for (_, id, name, icon) in self.pages {
            rows.push(Row::Page {
                id,
                name,
                icon,
                depth,
            });
        }

        let mut children: Vec<_> = self.children.into_iter().collect();
        children.sort_by_key(|(_, node)| node.priority());

        for (name, node) in children {
            let path = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{prefix}.{name}")
            };

            rows.push(Row::Chapter {
                name,
                path: path.clone(),
                depth,
            });

            node.flatten(&path, depth + 1, rows);
        }
    }
}

pub fn flatten<'a>(pages: impl Iterator<Item = &'a Page>) -> Vec<Row> {
    let mut root = Node::default();

    for page in pages {
        let node = page
            .chapter
            .as_deref()
            .filter(|chapter| !chapter.is_empty())
            .into_iter()
            .flat_map(|chapter| chapter.split('.'))
            .fold(&mut root, |node, segment| {
                node.children.entry(segment.to_string()).or_default()
            });

        node.pages
            .push((page.priority, page.id, page.name.clone(), page.kind.icon()));
    }

    let mut rows = Vec::new();

    root.flatten("", 0, &mut rows);

    rows
}
