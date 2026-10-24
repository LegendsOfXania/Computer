use computer_model::page::Page;
use std::collections::BTreeMap;

pub enum Row {
    Chapter {
        name: String,
        path: String,
        depth: usize,
    },
    Page {
        id: u64,
        name: String,
        depth: usize,
    },
}

#[derive(Default)]
struct Node {
    pages: Vec<(i32, u64, String)>,
    children: BTreeMap<String, Node>,
}

impl Node {
    fn priority(&self) -> i32 {
        self.pages
            .iter()
            .map(|p| p.0)
            .chain(self.children.values().map(Node::priority))
            .min()
            .unwrap_or(i32::MAX)
    }

    fn flatten(mut self, prefix: &str, depth: usize, rows: &mut Vec<Row>) {
        self.pages.sort_by_key(|p| p.0);
        for (_, id, name) in self.pages {
            rows.push(Row::Page { id, name, depth });
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
            .filter(|c| !c.is_empty())
            .into_iter()
            .flat_map(|c| c.split('.'))
            .fold(&mut root, |n, seg| {
                n.children.entry(seg.to_string()).or_default()
            });

        node.pages.push((page.priority, page.id, page.name.clone()));
    }

    let mut rows = Vec::new();
    root.flatten("", 0, &mut rows);
    rows
}
