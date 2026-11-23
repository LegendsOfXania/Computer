use crate::id::EntryId;
use crate::{Entry, Node, Value};
use indexmap::IndexSet;

impl Entry {
    #[must_use]
    pub fn new(id: EntryId, kind: String, version: u32) -> Self {
        Self {
            id: id.to_string(),
            kind,
            version,
            values: vec![Value::Struct(vec!["name".into()]), "".into()],
        }
    }

    #[must_use]
    pub fn id(&self) -> EntryId {
        EntryId::from(self.id.as_str())
    }

    #[must_use]
    pub fn root(&self) -> Node<'_> {
        Node::new(&self.values, 0)
    }

    #[must_use]
    pub fn get(&self, name: &str) -> Option<Node<'_>> {
        self.root().get(name)
    }

    #[must_use]
    pub fn label(&self) -> &str {
        match self.get("name").and_then(|node| node.value()) {
            Some(Value::String(name)) if !name.is_empty() => name,
            _ => &self.kind,
        }
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        Node::trees(&self.values) == Some(1)
            && matches!(self.values.first(), Some(Value::Struct(_)))
    }

    #[must_use]
    pub fn references(&self) -> IndexSet<EntryId> {
        self.root()
            .subtree()
            .iter()
            .filter_map(|value| match value {
                Value::Reference(id) => Some(EntryId::from(id.as_str())),
                _ => None,
            })
            .collect()
    }

    pub fn set(&mut self, name: &str, subtree: impl IntoIterator<Item = Value>) -> bool {
        if self.values.is_empty() {
            self.values.push(Value::Struct(Vec::new()));
        }

        if let Some(at) = self.get(name).map(|node| node.index()) {
            return self.replace(at, subtree);
        }

        let end = self.root().end();

        if !matches!(self.values.first(), Some(Value::Struct(_)))
            || !self.splice(end..end, subtree, 1)
        {
            return false;
        }

        if let Some(Value::Struct(names)) = self.values.first_mut() {
            names.push(name.to_owned());
        }

        true
    }

    pub fn replace(&mut self, at: usize, subtree: impl IntoIterator<Item = Value>) -> bool {
        let end = Node::new(&self.values, at).end();

        at < end && self.splice(at..end, subtree, 1)
    }

    pub fn insert(&mut self, at: usize, subtree: impl IntoIterator<Item = Value>) -> bool {
        let roots = match self.values.get(at) {
            Some(Value::List(_)) => 1,
            Some(Value::Map(_)) => 2,
            _ => return false,
        };

        let end = Node::new(&self.values, at).end();

        if !self.splice(end..end, subtree, roots) {
            return false;
        }

        if let Some(Value::List(count) | Value::Map(count)) = self.values.get_mut(at) {
            *count = count.saturating_add(1);
        }

        true
    }

    pub fn remove(&mut self, at: usize, index: usize) -> bool {
        let Some(range) = self.group(at, index) else {
            return false;
        };

        if let Some(Value::List(count) | Value::Map(count)) = self.values.get_mut(at) {
            *count -= 1;
        }

        self.values.drain(range);

        true
    }

    fn splice(
        &mut self,
        range: std::ops::Range<usize>,
        subtree: impl IntoIterator<Item = Value>,
        roots: usize,
    ) -> bool {
        let subtree: Vec<Value> = subtree.into_iter().collect();

        if Node::trees(&subtree) != Some(roots) || range.end > self.values.len() {
            return false;
        }

        self.values.splice(range, subtree).for_each(drop);

        true
    }

    fn group(&self, at: usize, index: usize) -> Option<std::ops::Range<usize>> {
        let node = Node::new(&self.values, at);

        let size = match node.value()? {
            Value::List(_) => 1,
            Value::Map(_) => 2,
            _ => return None,
        };

        let mut children = node.children().skip(index.checked_mul(size)?);
        let first = children.next()?;
        let last = if size == 1 { first } else { children.next()? };

        Some(first.index()..last.end())
    }
}

#[cfg(feature = "panel")]
use crate::EntryDefinition;

#[cfg(feature = "panel")]
impl TryFrom<&EntryDefinition> for Entry {
    type Error = String;

    fn try_from(definition: &EntryDefinition) -> Result<Self, String> {
        Ok(Self::new(
            EntryId::generate()?,
            definition.kind.clone(),
            definition.version,
        ))
    }
}
