use std::{
    collections::BTreeSet,
    fmt::Display,
    fs::{self, File},
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
};

use arc_swap::ArcSwap;
use computer_model::{
    entry::Entry,
    key::EntryKey,
    page::Page,
    protocol::event::Event,
    Library,
};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use crate::data;

static LIVE: LazyLock<ArcSwap<Library>> =
    LazyLock::new(|| ArcSwap::from_pointee(Library::default()));

static DEV: Mutex<Option<Dev>> = Mutex::new(None);

struct Dev {
    library: Library,
    dirty: BTreeSet<u64>,
}

#[derive(Deserialize)]
struct FileData {
    page: Page,
    entries: Vec<Entry>,
}

#[derive(Serialize)]
struct FileRef<'a> {
    page: &'a Page,
    entries: Vec<&'a Entry>,
}

fn err(error: impl Display) -> String {
    error.to_string()
}

fn path_err(path: &Path, error: impl Display) -> String {
    format!("{}: {error}", path.display())
}

fn library_dir(name: &str) -> Result<PathBuf, String> {
    let data = data::get_data_folder().ok_or("no data folder")?;

    Ok(Path::new(data).join("pages").join(name))
}

fn live_dir() -> Result<PathBuf, String> {
    library_dir("live")
}

fn dev_dir() -> Result<PathBuf, String> {
    library_dir("dev")
}

fn page_file(dir: &Path, page_id: u64) -> PathBuf {
    dir.join(format!("{page_id}.json"))
}

#[allow(dead_code)]
pub fn live() -> Arc<Library> {
    LIVE.load_full()
}

fn with_dev<T>(f: impl FnOnce(&mut Dev) -> Result<T, String>) -> Result<T, String> {
    let mut guard = DEV.lock();

    f(guard.as_mut().ok_or("dev library is not open: no panel has connected.")?)
}

/// Applies a change to the dev library in memory. The touched pages are only
/// marked dirty: `flush_dirty` writes them (at most once per second, and
/// before a push or when the plugin unloads), so an edit never waits for the
/// disk on the tick.
fn edit_dev<T>(
    f: impl FnOnce(&mut Library) -> Result<(T, Vec<u64>), String>,
) -> Result<T, String> {
    with_dev(|dev| {
        let (value, touched) = f(&mut dev.library)?;

        dev.dirty.extend(touched);

        Ok(value)
    })
}

/// Writes the pages edited since the last call. A failure is only logged: the
/// pages stay dirty and are written again on the next call.
pub fn flush_dirty() {
    let mut guard = DEV.lock();

    let Some(dev) = guard.as_mut() else {
        return;
    };

    if dev.dirty.is_empty() {
        return;
    }

    if let Err(error) = flush(dev) {
        tracing::error!("Could not save the dev library, will retry: {error}");
    }
}

fn load(dir: &Path) -> Result<Library, String> {
    let read_dir = match fs::read_dir(dir) {
        Ok(read_dir) => read_dir,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Library::default()),
        Err(error) => return Err(err(error)),
    };

    let mut files = Vec::new();

    for item in read_dir {
        let path = match item {
            Ok(item) => item.path(),

            Err(error) => {
                tracing::error!("Could not list a file of {}: {error}", dir.display());
                continue;
            }
        };

        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        // One damaged page must not make the whole library unusable: it is
        // set aside (never deleted) and the other pages still load.
        match read_page_file(&path) {
            Ok(file) => files.push(file),

            Err(error) => {
                tracing::error!("Skipped a damaged page file: {error}");
                set_aside(&path);
            }
        }
    }

    files.sort_by_key(|file| file.page.id);

    let mut library = Library::default();

    for FileData { mut page, entries } in files {
        let keys: BTreeSet<EntryKey> = entries.iter().map(|entry| entry.key).collect();

        library
            .entries
            .extend(entries.into_iter().map(|entry| (entry.key, entry)));

        // A page and its entries must agree: forget the keys that point to
        // nothing, and keep (rather than lose) the entries nobody lists.
        page.entries.retain(|key| keys.contains(key));

        for key in keys {
            if !page.entries.contains(&key) {
                tracing::warn!("Entry {key} was not listed by its page {}, re-attached", page.id);
                page.entries.push(key);
            }
        }

        library.pages.insert(page.id, page);
    }

    Ok(library)
}

fn read_page_file(path: &Path) -> Result<FileData, String> {
    let content = fs::read(path).map_err(|error| path_err(path, error))?;

    serde_json::from_slice(&content).map_err(|error| path_err(path, error))
}

/// Renames a damaged page file to `<name>.json.broken`, so it is ignored by
/// the next loads but can still be inspected or repaired by hand.
fn set_aside(path: &Path) {
    let broken = path.with_extension("json.broken");

    match fs::rename(path, &broken) {
        Ok(()) => tracing::error!("It was moved to {}", broken.display()),
        Err(error) => tracing::error!("It could not be moved aside: {error}"),
    }
}

fn encode_page(library: &Library, page_id: u64) -> Result<Vec<u8>, String> {
    let page = library
        .pages
        .get(&page_id)
        .ok_or_else(|| format!("page {page_id} not found"))?;

    let entries = page
        .entries
        .iter()
        .filter_map(|key| library.entries.get(key))
        .collect();

    serde_json::to_vec_pretty(&FileRef { page, entries }).map_err(err)
}

/// Writes `data` atomically (temporary file + rename). `sync` also waits for
/// the disk, which only matters for the live library.
fn save_file(path: &Path, data: &[u8], sync: bool) -> Result<(), String> {
    let temporary = path.with_extension("json.tmp");

    let result = (|| -> Result<(), String> {
        let mut file = File::create(&temporary).map_err(err)?;

        file.write_all(data).map_err(err)?;
        if sync {
            file.sync_all().map_err(err)?;
        }

        fs::rename(&temporary, path).map_err(err)
    })();

    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }

    result
}

fn remove_file(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(err(error)),
    }
}

fn remove_directory(dir: &Path) -> Result<(), String> {
    match fs::remove_dir_all(dir) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(err(error)),
    }
}

fn sync_page(dir: &Path, library: &Library, page_id: u64) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(err)?;

    let path = page_file(dir, page_id);

    if library.pages.contains_key(&page_id) {
        save_file(&path, &encode_page(library, page_id)?, false)?;
    } else {
        remove_file(&path)?;
    }

    Ok(())
}

fn write_library(dir: &Path, library: &Library, sync: bool) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(err)?;

    for id in library.pages.keys() {
        save_file(&page_file(dir, *id), &encode_page(library, *id)?, sync)?;
    }

    Ok(())
}

fn recover_live(live: &Path) -> Result<(), String> {
    let old = live.with_extension("old");
    let new = live.with_extension("new");

    if !live.exists() && old.exists() {
        fs::rename(&old, live).map_err(err)?;
    }

    remove_directory(&new)?;

    if live.exists() {
        remove_directory(&old)?;
    }

    Ok(())
}

pub fn init_live() {
    let result = live_dir().and_then(|dir| {
        recover_live(&dir)?;
        fs::create_dir_all(&dir).map_err(err)?;
        reload()
    });

    match result {
        Ok(()) => tracing::info!("Live library loaded"),
        Err(error) => tracing::error!("Could not load the live library: {error}"),
    }
}

pub fn reload() -> Result<(), String> {
    LIVE.store(Arc::new(load(&live_dir()?)?));

    Ok(())
}

pub fn push() -> Result<(), String> {
    let mut guard = DEV.lock();
    let dev = guard.as_mut().ok_or("dev library is not open: no panel has connected.")?;

    // Keep the dev folder in step with what is pushed.
    if let Err(error) = flush(dev) {
        tracing::error!("Could not save the dev library before the push: {error}");
    }

    let live = live_dir()?;
    let new = live.with_extension("new");
    let old = live.with_extension("old");

    remove_directory(&new)?;
    remove_directory(&old)?;

    if let Err(error) = write_library(&new, &dev.library, true) {
        let _ = remove_directory(&new);
        return Err(error);
    }

    if live.exists() {
        if let Err(error) = fs::rename(&live, &old) {
            let _ = remove_directory(&new);
            return Err(err(error));
        }
    }

    if let Err(error) = fs::rename(&new, &live) {
        if old.exists() && !live.exists() {
            let _ = fs::rename(&old, &live);
        }

        let _ = remove_directory(&new);

        return Err(err(error));
    }

    let _ = remove_directory(&old);

    // The dev library is kept as is: it now equals live, and the panels that
    // are still connected can keep editing it.
    drop(guard);

    reload()
}

pub fn init_dev() -> Result<Library, String> {
    let mut guard = DEV.lock();

    if let Some(dev) = guard.as_ref() {
        return Ok(dev.library.clone());
    }

    let dir = dev_dir()?;

    let library = if dir.exists() {
        load(&dir)?
    } else {
        let library = (**LIVE.load()).clone();

        write_library(&dir, &library, false)?;

        library
    };

    *guard = Some(Dev {
        library: library.clone(),
        dirty: BTreeSet::new(),
    });

    Ok(library)
}

fn flush(dev: &mut Dev) -> Result<(), String> {
    let dir = dev_dir()?;

    while let Some(&id) = dev.dirty.first() {
        sync_page(&dir, &dev.library, id)?;
        dev.dirty.pop_first();
    }

    Ok(())
}

fn detach(library: &mut Library, key: &EntryKey) -> Vec<u64> {
    library
        .pages
        .values_mut()
        .filter(|page| page.entries.contains(key))
        .map(|page| {
            page.entries.retain(|entry| entry != key);
            page.id
        })
        .collect()
}

fn pages_containing(library: &Library, key: &EntryKey) -> Vec<u64> {
    library
        .pages
        .values()
        .filter(|page| page.entries.contains(key))
        .map(|page| page.id)
        .collect()
}

pub fn create_page(page: Page) -> Result<Event, String> {
    edit_dev(|library| {
        let id = page.id;

        if library.pages.contains_key(&id) {
            return Err(format!("page {id} already exists"));
        }

        // The entries of a page are owned by the engine: a new page is empty.
        let page = Page {
            entries: Vec::new(),
            ..page
        };

        library.pages.insert(id, page.clone());

        Ok((Event::PageCreated { page }, vec![id]))
    })
}

pub fn edit_page(page: Page) -> Result<Event, String> {
    edit_dev(|library| {
        // Destructured on purpose: a new field on `Page` will not compile
        // until it is decided whether a client may edit it.
        let Page {
            id,
            name,
            kind,
            priority,
            chapter,
            entries: _,
        } = page;

        let existing = library
            .pages
            .get_mut(&id)
            .ok_or_else(|| format!("page {id} not found: it may have been deleted"))?;

        existing.name = name;
        existing.kind = kind;
        existing.priority = priority;
        existing.chapter = chapter;

        // `entries` is kept as it is in the library, and the event carries the
        // resulting page, not the one sent by the client.
        Ok((Event::PageUpdated { page: existing.clone() }, vec![id]))
    })
}

pub fn delete_page(page_id: u64) -> Result<Event, String> {
    edit_dev(|library| {
        let page = library
            .pages
            .shift_remove(&page_id)
            .ok_or_else(|| format!("page {page_id} not found: it may already be deleted"))?;

        for key in &page.entries {
            library.entries.shift_remove(key);
        }

        Ok((Event::PageDeleted { page_id }, vec![page_id]))
    })
}

pub fn create_entry(entry: Entry) -> Result<Event, String> {
    edit_dev(|library| {
        let key = entry.key;
        let page_id = key.page_id();

        if library.entries.contains_key(&key) {
            return Err(format!("entry {key} already exists"));
        }

        let page = library
            .pages
            .get_mut(&page_id)
            .ok_or_else(|| format!("page {page_id} not found: it may have been deleted"))?;

        page.entries.push(key);
        library.entries.insert(key, entry.clone());

        Ok((Event::EntryCreated { entry }, vec![page_id]))
    })
}

pub fn edit_entry(entry: Entry) -> Result<Event, String> {
    edit_dev(|library| {
        let key = entry.key;

        let existing = library
            .entries
            .get_mut(&key)
            .ok_or_else(|| format!("entry {key} not found: it may have been deleted"))?;

        *existing = entry.clone();

        Ok((Event::EntryUpdated { entry }, pages_containing(library, &key)))
    })
}

pub fn delete_entry(key: EntryKey) -> Result<Event, String> {
    edit_dev(|library| {
        if library.entries.shift_remove(&key).is_none() {
            return Err(format!("entry {key} not found: it may already be deleted"));
        }

        Ok((Event::EntryDeleted { key }, detach(library, &key)))
    })
}

pub fn move_entry(key: EntryKey, page_id: u64) -> Result<Event, String> {
    edit_dev(|library| {
        if !library.entries.contains_key(&key) {
            return Err(format!("entry {key} not found: it may have been deleted"));
        }

        if !library.pages.contains_key(&page_id) {
            return Err(format!("page {page_id} not found: it may have been deleted"));
        }

        let mut touched = detach(library, &key);

        if let Some(page) = library.pages.get_mut(&page_id) {
            page.entries.push(key);
        }

        touched.push(page_id);

        Ok((Event::EntryMoved { key, page_id }, touched))
    })
}
