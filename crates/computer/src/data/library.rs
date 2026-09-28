use std::{
    fs,
    io::ErrorKind,
    path::Path,
    sync::Mutex,
};

use computer_model::{entry::Entry, key::EntryKey, page::Page, Library};
use serde::{Deserialize, Serialize};

use crate::data;

const NOT_OPEN: &str = "dev library is not open: no panel has connected.";

static LIVE: Mutex<Library> = Mutex::new(Library {
    pages: Vec::new(),
    entries: Vec::new(),
});

static DEV: Mutex<Option<Library>> = Mutex::new(None);

#[derive(Serialize, Deserialize)]
struct File {
    page: Page,
    entries: Vec<Entry>,
}

fn live_dir() -> Result<std::path::PathBuf, String> {
    let data = data::get_data_folder().ok_or("no data folder")?;
    Ok(Path::new(data).join("pages/live"))
}

fn dev_dir() -> Result<std::path::PathBuf, String> {
    let data = data::get_data_folder().ok_or("no data folder")?;
    Ok(Path::new(data).join("pages/dev"))
}

fn load(dir: &Path) -> Result<Library, String> {
    let mut library = Library {
        pages: Vec::new(),
        entries: Vec::new(),
    };

    let read_dir = match fs::read_dir(dir) {
        Ok(read_dir) => read_dir,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(library),
        Err(error) => return Err(error.to_string()),
    };

    for item in read_dir {
        let path = item.map_err(|e| e.to_string())?.path();

        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let content =
            fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;

        let file: File = serde_json::from_slice(&content)
            .map_err(|e| format!("{}: {e}", path.display()))?;

        library.pages.push(file.page);
        library.entries.extend(file.entries);
    }

    library.pages.sort_by_key(|page| page.id);

    Ok(library)
}

fn save_page(dir: &Path, library: &Library, page_id: u64) -> Result<(), String> {
    let page = library
        .pages
        .iter()
        .find(|page| page.id == page_id)
        .ok_or_else(|| format!("page {page_id} not found"))?;

    let entries = page
        .entries
        .iter()
        .filter_map(|key| library.entries.iter().find(|entry| entry.key == *key))
        .cloned()
        .collect();

    let file = File {
        page: page.clone(),
        entries,
    };

    fs::create_dir_all(dir).map_err(|e| e.to_string())?;

    let json = serde_json::to_vec_pretty(&file).map_err(|e| e.to_string())?;

    fs::write(dir.join(format!("{page_id}.json")), json)
        .map_err(|e| e.to_string())
}

fn save_library(dir: &Path, library: &Library) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;

    for page in &library.pages {
        save_page(dir, library, page.id)?;
    }

    Ok(())
}

pub fn init_live() {
    match reload() {
        Ok(()) => tracing::info!("Live library loaded"),
        Err(error) => tracing::error!("Could not load the live library: {error}"),
    }
}

pub fn reload() -> Result<(), String> {
    let library = load(&live_dir()?)?;

    *LIVE.lock().map_err(|e| e.to_string())? = library;

    Ok(())
}

pub fn push() -> Result<(), String> {
    let guard = DEV.lock().map_err(|e| e.to_string())?;
    let dev = guard.as_ref().ok_or(NOT_OPEN)?;

    let dir = live_dir()?;

    if dir.exists() {
        fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
    }

    save_library(&dir, dev)?;

    drop(guard);

    reload()
}

pub fn init_dev() -> Result<Library, String> {
    let mut dev = DEV.lock().map_err(|e| e.to_string())?;

    if let Some(library) = dev.as_ref() {
        return Ok(library.clone());
    }

    let dir = dev_dir()?;

    let library = if dir.exists() {
        load(&dir)?
    } else {
        let library = LIVE.lock().map_err(|e| e.to_string())?.clone();

        save_library(&dir, &library)?;

        library
    };

    *dev = Some(library.clone());

    Ok(library)
}

pub fn create_page(page: Page) -> Result<(), String> {
    let mut dev = DEV.lock().map_err(|e| e.to_string())?;
    let library = dev.as_mut().ok_or(NOT_OPEN)?;

    let id = page.id;

    if library.pages.iter().any(|page| page.id == id) {
        return Err(format!("page {id} already exists"));
    }

    library.pages.push(page);

    save_page(&dev_dir()?, library, id)
}

pub fn edit_page(page: Page) -> Result<(), String> {
    let mut dev = DEV.lock().map_err(|e| e.to_string())?;
    let library = dev.as_mut().ok_or(NOT_OPEN)?;

    let id = page.id;

    let existing = library
        .pages
        .iter_mut()
        .find(|page| page.id == id)
        .ok_or_else(|| format!("page {id} not found"))?;

    *existing = page;

    save_page(&dev_dir()?, library, id)
}

pub fn delete_page(page_id: u64) -> Result<(), String> {
    let mut dev = DEV.lock().map_err(|e| e.to_string())?;
    let library = dev.as_mut().ok_or(NOT_OPEN)?;

    let Some(index) = library.pages.iter().position(|page| page.id == page_id) else {
        return Ok(());
    };

    let page = library.pages.remove(index);

    library
        .entries
        .retain(|entry| !page.entries.contains(&entry.key));

    match fs::remove_file(dev_dir()?.join(format!("{page_id}.json"))) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(error.to_string()),
    }

    Ok(())
}

pub fn create_entry(entry: Entry) -> Result<(), String> {
    let mut dev = DEV.lock().map_err(|e| e.to_string())?;
    let library = dev.as_mut().ok_or(NOT_OPEN)?;

    let key = entry.key;

    if library.entries.iter().any(|entry| entry.key == key) {
        return Err(format!("entry {:?} already exists", key));
    }

    let page_id = key.page_id();

    let page = library
        .pages
        .iter_mut()
        .find(|page| page.id == page_id)
        .ok_or_else(|| format!("page {page_id} not found"))?;

    page.entries.push(key);
    library.entries.push(entry);

    save_page(&dev_dir()?, library, page_id)
}

pub fn edit_entry(entry: Entry) -> Result<(), String> {
    let mut dev = DEV.lock().map_err(|e| e.to_string())?;
    let library = dev.as_mut().ok_or(NOT_OPEN)?;

    let key = entry.key;

    let existing = library
        .entries
        .iter_mut()
        .find(|entry| entry.key == key)
        .ok_or_else(|| format!("entry {:?} not found", key))?;

    *existing = entry;

    save_page(&dev_dir()?, library, key.page_id())
}

pub fn delete_entry(key: EntryKey) -> Result<(), String> {
    let mut dev = DEV.lock().map_err(|e| e.to_string())?;
    let library = dev.as_mut().ok_or(NOT_OPEN)?;

    library.entries.retain(|entry| entry.key != key);

    let mut pages = Vec::new();

    for page in &mut library.pages {
        if page.entries.contains(&key) {
            page.entries.retain(|entry| *entry != key);
            pages.push(page.id);
        }
    }

    let dir = dev_dir()?;

    for page_id in pages {
        save_page(&dir, library, page_id)?;
    }

    Ok(())
}

pub fn move_entry(key: EntryKey, page_id: u64) -> Result<(), String> {
    let mut dev = DEV.lock().map_err(|e| e.to_string())?;
    let library = dev.as_mut().ok_or(NOT_OPEN)?;

    if !library.pages.iter().any(|page| page.id == page_id) {
        return Err(format!("page {page_id} not found"));
    }

    let mut touched = Vec::new();

    for page in &mut library.pages {
        if page.entries.contains(&key) {
            page.entries.retain(|entry| *entry != key);
            touched.push(page.id);
        }
    }

    library
        .pages
        .iter_mut()
        .find(|page| page.id == page_id)
        .unwrap()
        .entries
        .push(key);

    touched.push(page_id);
    touched.sort_unstable();
    touched.dedup();

    let dir = dev_dir()?;

    for page_id in touched {
        save_page(&dir, library, page_id)?;
    }

    Ok(())
}