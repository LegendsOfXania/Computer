#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PageType {
    Sequence,
    Manifest,
    Static,
}

impl PageType {
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sequence => "sequence",
            Self::Manifest => "manifest",
            Self::Static => "static",
        }
    }

    #[inline]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "sequence" => Some(Self::Sequence),
            "manifest" => Some(Self::Manifest),
            "static" => Some(Self::Static),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageData {
    id: String,
    name: String,
    page_type: PageType,
    priority: u32,
}

impl PageData {
    #[inline]
    pub fn new(id: impl Into<String>, name: impl Into<String>, page_type: PageType, priority: u32) -> Self {
        Self { id: id.into(), name: name.into(), page_type, priority }
    }
    #[inline]
    pub fn id(&self) -> &str { &self.id }
    #[inline]
    pub fn name(&self) -> &str { &self.name }
    #[inline]
    pub const fn page_type(&self) -> PageType { self.page_type }
    #[inline]
    pub const fn priority(&self) -> u32 { self.priority }
}
