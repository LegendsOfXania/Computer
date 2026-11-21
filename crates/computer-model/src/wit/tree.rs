use super::{Field, FieldKind, Value};

pub trait Tree {
    fn arity(&self) -> usize;
}

impl Tree for Value {
    fn arity(&self) -> usize {
        match self {
            Self::Struct(names) | Self::Variant((_, names)) => names.len(),
            Self::List(count) => *count as usize,
            Self::Map(count) => (*count as usize).saturating_mul(2),
            _ => 0,
        }
    }
}

impl Tree for Field {
    fn arity(&self) -> usize {
        match &self.kind {
            FieldKind::Struct(count) => *count as usize,
            FieldKind::List => 1,
            FieldKind::Map => 2,
            FieldKind::Variant(cases) => cases.iter().map(|(_, count)| *count as usize).sum(),
            _ => 0,
        }
    }
}

pub struct Cursor<'a, T> {
    items: &'a [T],
    pos: usize,
}

pub type Node<'a> = Cursor<'a, Value>;
pub type FieldNode<'a> = Cursor<'a, Field>;

impl<T> Clone for Cursor<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Cursor<'_, T> {}

impl<'a, T: Tree> Cursor<'a, T> {
    pub(super) const fn new(items: &'a [T], pos: usize) -> Self {
        Self { items, pos }
    }

    pub fn forest(items: &'a [T]) -> impl Iterator<Item = Self> + 'a {
        std::iter::successors(Some(Self::new(items, 0)), |tree| {
            Some(Self::new(tree.items, tree.end()))
        })
            .take_while(|tree| tree.value().is_some())
    }

    #[must_use]
    pub const fn index(&self) -> usize {
        self.pos
    }

    #[must_use]
    pub fn value(&self) -> Option<&'a T> {
        self.items.get(self.pos)
    }

    #[must_use]
    pub fn end(&self) -> usize {
        let mut pending = 1_usize;
        let mut end = self.pos;

        for item in self.items.iter().skip(self.pos) {
            pending = pending.saturating_add(item.arity()) - 1;
            end += 1;

            if pending == 0 {
                break;
            }
        }

        end
    }

    #[must_use]
    pub fn trees(items: &[T]) -> Option<usize> {
        let (mut pending, mut roots) = (0_usize, 0);

        for item in items {
            if pending == 0 {
                roots += 1;
                pending = 1;
            }

            pending = pending.saturating_add(item.arity()) - 1;
        }

        (pending == 0).then_some(roots)
    }

    #[must_use]
    pub fn subtree(&self) -> &'a [T] {
        self.items.get(self.pos..self.end()).unwrap_or_default()
    }

    pub fn children(&self) -> impl Iterator<Item = Self> + 'a {
        let cursor = *self;
        let mut previous: Option<Self> = None;

        (0..self.value().map_or(0, T::arity)).map_while(move |_| {
            let pos = previous.map_or(cursor.pos + 1, |child| child.end());
            let child = Self { pos, ..cursor };

            child.value()?;
            previous = Some(child);

            Some(child)
        })
    }
}