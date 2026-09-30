use std::{
    collections::BTreeSet,
    fmt::Display,
    fs::{self, File},
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
};

use arc_swap::ArcSwap;
use computer_model::{entry::Entry, key::EntryKey, page::Page, Library};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use crate::data;

const NOT_OPEN: &str = "dev library is not open: no panel has connected.";

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

    f(guard.as_mut().ok_or(NOT_OPEN)?)
}

fn edit_dev(f: impl FnOnce(&mut Library) -> Result<Vec<u64>, String>) -> Result<(), String> {
    with_dev(|dev| {
        let touched = f(&mut dev.library)?;

        dev.dirty.extend(touched);

        flush(dev)
    })
}

fn load(dir: &Path) -> Result<Library, String> {
    let read_dir = match fs::read_dir(dir) {
        Ok(read_dir) => read_dir,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Library::default()),
        Err(error) => return Err(err(error)),
    };

    let mut files = Vec::new();

    for item in read_dir {
        let path = item.map_err(err)?.path();

        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let content = fs::read(&path).map_err(|error| path_err(&path, error))?;

        let file: FileData =
            serde_json::from_slice(&content).map_err(|error| path_err(&path, error))?;

        files.push(file);
    }

    files.sort_by_key(|file| file.page.id);

    let mut library = Library::default();

    for file in files {
        library
            .entries
            .extend(file.entries.into_iter().map(|entry| (entry.key, entry)));

        library.pages.insert(file.page.id, file.page);
    }

    Ok(library)
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

fn save_file(path: &Path, data: &[u8]) -> Result<(), String> {
    let temporary = path.with_extension("json.tmp");

    let result = (|| -> Result<(), String> {
        let mut file = File::create(&temporary).map_err(err)?;

        file.write_all(data).map_err(err)?;
        file.sync_all().map_err(err)?;

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
        save_file(&path, &encode_page(library, page_id)?)?;
    } else {
        remove_file(&path)?;
    }

    Ok(())
}

fn write_library(dir: &Path, library: &Library) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(err)?;

    for id in library.pages.keys() {
        save_file(&page_file(dir, *id), &encode_page(library, *id)?)?;
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
    let dev = guard.as_ref().ok_or(NOT_OPEN)?;

    let live = live_dir()?;
    let new = live.with_extension("new");
    let old = live.with_extension("old");

    remove_directory(&new)?;
    remove_directory(&old)?;

    if let Err(error) = write_library(&new, &dev.library) {
        let _ = remove_directory(&new);
        return Err(error);
    }

    if live.exists() {
        fs::rename(&live, &old).map_err(err)?;
    }

    if let Err(error) = fs::rename(&new, &live) {
        if old.exists() && !live.exists() {
            let _ = fs::rename(&old, &live);
        }

        let _ = remove_directory(&new);

        return Err(err(error));
    }

    let _ = remove_directory(&old);

    *guard = None;

    if let Err(error) = dev_dir().and_then(|dir| remove_directory(&dir)) {
        tracing::warn!("Could not remove dev library after push: {error}");
    }

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

        write_library(&dir, &library)?;

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

pub fn create_page(page: Page) -> Result<(), String> {
    edit_dev(|library| {
        let id = page.id;

        if library.pages.contains_key(&id) {
            return Err(format!("page {id} already exists"));
        }

        library.pages.insert(id, page);

        Ok(vec![id])
    })
}

pub fn edit_page(page: Page) -> Result<(), String> {
    edit_dev(|library| {
        let id = page.id;

        let existing = library
            .pages
            .get_mut(&id)
            .ok_or_else(|| format!("page {id} not found"))?;

        *existing = page;

        Ok(vec![id])
    })
}

pub fn delete_page(page_id: u64) -> Result<(), String> {
    edit_dev(|library| {
        let Some(page) = library.pages.shift_remove(&page_id) else {
            return Ok(Vec::new());
        };

        for key in &page.entries {
            library.entries.shift_remove(key);
        }

        Ok(vec![page_id])
    })
}

pub fn create_entry(entry: Entry) -> Result<(), String> {
    edit_dev(|library| {
        let key = entry.key;
        let page_id = key.page_id();

        if library.entries.contains_key(&key) {
            return Err(format!("entry {key:?} already exists"));
        }

        let page = library
            .pages
            .get_mut(&page_id)
            .ok_or_else(|| format!("page {page_id} not found"))?;

        page.entries.push(key);
        library.entries.insert(key, entry);

        Ok(vec![page_id])
    })
}

pub fn edit_entry(entry: Entry) -> Result<(), String> {
    edit_dev(|library| {
        let key = entry.key;

        let existing = library
            .entries
            .get_mut(&key)
            .ok_or_else(|| format!("entry {key:?} not found"))?;

        *existing = entry;

        Ok(pages_containing(library, &key))
    })
}

pub fn delete_entry(key: EntryKey) -> Result<(), String> {
    edit_dev(|library| {
        library.entries.shift_remove(&key);

        Ok(detach(library, &key))
    })
}

pub fn move_entry(key: EntryKey, page_id: u64) -> Result<(), String> {
    edit_dev(|library| {
        if !library.entries.contains_key(&key) {
            return Err(format!("entry {key:?} not found"));
        }

        if !library.pages.contains_key(&page_id) {
            return Err(format!("page {page_id} not found"));
        }

        let mut touched = detach(library, &key);

        if let Some(page) = library.pages.get_mut(&page_id) {
            page.entries.push(key);
        }

        touched.push(page_id);

        Ok(touched)
    })
}