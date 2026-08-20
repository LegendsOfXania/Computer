use model::{EntryData, PageData, PageType};
use runtime::{Entry, EntryKey, Library, Page, Ref};

#[derive(Debug)]
struct TestEntry {
    id: String,
    data: EntryData,
}

impl TestEntry {
    fn new(id: &str, entry_type: &str) -> Self {
        Self {
            id: id.into(),
            data: EntryData::empty(entry_type),
        }
    }
}

impl Entry for TestEntry {
    fn id(&self) -> &str { &self.id }
    fn data(&self) -> &EntryData { &self.data }
}

#[derive(Debug)]
struct OtherEntry {
    id: String,
    data: EntryData,
}

impl Entry for OtherEntry {
    fn id(&self) -> &str { &self.id }
    fn data(&self) -> &EntryData { &self.data }
}

#[test]
fn registry_finds_typed_entries() {
    let registry = runtime::EntryStore::new();
    let key = EntryKey::new("page", "entry");

    registry.insert(key.clone(), TestEntry::new("entry", "dialogue"));

    assert_eq!(
        registry.find::<TestEntry>(&key).unwrap().entry_type(),
        "dialogue",
    );
    assert!(registry.find::<OtherEntry>(&key).is_none());
}

#[test]
fn library_indexes_entries_by_page_and_entry_id() {
    let library = Library::new();
    let mut page = Page::new(PageData::new("page", "Page", PageType::Sequence, 0));
    page.add_entry(TestEntry::new("entry", "dialogue"));
    library.add_page(page);

    let key = EntryKey::new("page", "entry");
    assert_eq!(library.find::<TestEntry>(&key).unwrap().id(), "entry");
    assert!(library.page("page").is_some());
}

#[test]
fn replacing_a_page_replaces_its_entries() {
    let library = Library::new();

    let mut first = Page::new(PageData::new("page", "First", PageType::Sequence, 0));
    first.add_entry(TestEntry::new("first", "dialogue"));
    library.add_page(first);

    let mut second = Page::new(PageData::new("page", "Second", PageType::Sequence, 0));
    second.add_entry(TestEntry::new("second", "dialogue"));
    library.add_page(second);

    assert!(library.find::<TestEntry>(&EntryKey::new("page", "first")).is_none());
    assert!(library.find::<TestEntry>(&EntryKey::new("page", "second")).is_some());
}

#[test]
fn references_use_qualified_keys() {
    let reference = Ref::<TestEntry>::new("page", "entry");
    assert_eq!(reference.key().unwrap().to_string(), "page:entry");
    assert!(Ref::<TestEntry>::empty().key().is_none());
}
