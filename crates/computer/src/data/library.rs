use std::{
    collections::HashSet,
    fmt::Display,
    fs::{self, File},
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
};

use arc_swap::ArcSwap;
use computer_model::{entry::Entry, id::{EntryId, PageId}, page::Page, protocol::event::Event, Library};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use crate::data;

static LIVE: LazyLock<ArcSwap<Library>> =
    LazyLock::new(|| ArcSwap::from_pointee(Library::default()));

static DEV: Mutex<Option<Dev>> = Mutex::new(None);

struct Dev {
    library: Library,
    dirty: Vec<PageId>,
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

fn page_file(dir: &Path, id: PageId) -> PathBuf {
    dir.join(format!("{id}.json"))
}

#[allow(dead_code)]
pub fn live() -> Arc<Library> {
    LIVE.load_full()
}

fn with_dev<T>(f: impl FnOnce(&mut Dev) -> Result<T, String>) -> Result<T, String> {
    let mut guard = DEV.lock();

    f(guard
        .as_mut()
        .ok_or("dev library is not open: no panel has connected.")?)
}

pub fn apply(event: Event) -> Result<Event, String> {
    with_dev(|dev| {
        for id in dev.library.apply(&event)? {
            if !dev.dirty.contains(&id) {
                dev.dirty.push(id);
            }
        }

        Ok(event)
    })
}

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
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Ok(Library::default());
        }
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

        match read_page_file(&path) {
            Ok(file) => files.push(file),
            Err(error) => {
                tracing::error!("Skipped a damaged page file: {error}");
                set_aside(&path);
            }
        }
    }

    files.sort_by_key(|file: &FileData| file.page.id);

    let mut library = Library::default();

    for FileData { mut page, entries } in files {
        let page_id = page.id;

        if library.pages.contains_key(&page_id) {
            return Err(format!("Duplicate page ID `{}` in library files", page_id));
        }

        let ids: Vec<EntryId> = entries.iter().map(|entry| entry.id).collect();

        let unique_ids: HashSet<EntryId> = ids.iter().copied().collect();

        if unique_ids.len() != ids.len() {
            return Err(format!("Page {} contains duplicate entry IDs", page.id));
        }

        for entry in &entries {
            if library.entries.contains_key(&entry.id) {
                return Err(format!(
                    "Entry ID {} occurs in more than one page file",
                    entry.id
                ));
            }
        }

        page.entries.retain(|id| unique_ids.contains(id));

        for id in &ids {
            if !page.entries.contains(id) {
                tracing::warn!(
                    "Entry {} was not listed by its page {}, re-attached",
                    id,
                    page.id
                );

                page.entries.push(*id);
            }
        }

        library
            .entries
            .extend(entries.into_iter().map(|entry| (entry.id, entry)));

        library.pages.insert(page_id, page);
    }

    Ok(library)
}

fn read_page_file(path: &Path) -> Result<FileData, String> {
    let content = fs::read(path).map_err(|error| path_err(path, error))?;

    serde_json::from_slice(&content).map_err(|error| path_err(path, error))
}

fn set_aside(path: &Path) {
    let broken = path.with_extension("json.broken");

    match fs::rename(path, &broken) {
        Ok(()) => tracing::error!("It was moved to {}", broken.display()),
        Err(error) => tracing::error!("It could not be moved aside: {error}"),
    }
}

fn encode_page(library: &Library, page_id: PageId) -> Result<Vec<u8>, String> {
    let page = library
        .pages
        .get(&page_id)
        .ok_or_else(|| format!("Page {} not found", page_id))?;

    let entries = page
        .entries
        .iter()
        .filter_map(|id| library.entries.get(id))
        .collect();

    serde_json::to_vec_pretty(&FileRef { page, entries }).map_err(err)
}

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

fn sync_page(dir: &Path, library: &Library, page_id: PageId) -> Result<(), String> {
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

    for id in library.pages.keys().copied() {
        save_file(&page_file(dir, id), &encode_page(library, id)?, sync)?;
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

    let dev = guard
        .as_mut()
        .ok_or("dev library is not open: no panel has connected.")?;

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

    if live.exists()
        && let Err(error) = fs::rename(&live, &old)
    {
        let _ = remove_directory(&new);
        return Err(err(error));
    }

    if let Err(error) = fs::rename(&new, &live) {
        if old.exists() && !live.exists() {
            let _ = fs::rename(&old, &live);
        }

        let _ = remove_directory(&new);
        return Err(err(error));
    }

    let _ = remove_directory(&old);

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
        dirty: Vec::new(),
    });

    Ok(library)
}

fn flush(dev: &mut Dev) -> Result<(), String> {
    let dir = dev_dir()?;

    while let Some(id) = dev.dirty.first().copied() {
        sync_page(&dir, &dev.library, id)?;
        dev.dirty.retain(|pending| pending != &id);
    }

    Ok(())
}
