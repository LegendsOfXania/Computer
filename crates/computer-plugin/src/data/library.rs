use std::{fs, io::ErrorKind, path::Path, sync::Mutex};

use computer_model::{entry::Entry, page::Page};
use serde::{Deserialize, Serialize};

enum Env {
    Dev,
    Live,
}

impl Env {
    fn dir(self) -> &'static str {
        match self {
            Env::Dev => "dev",
            Env::Live => "live",
        }
    }
}

#[derive(Deserialize, Serialize)]
struct File {
    page: Page,
    entries: Vec<Entry>,
}

static LIBRARY: Mutex<Option<(Vec<Page>, Vec<Entry>)>> = Mutex::new(None);

pub fn init_library() {
    match load() {
        Ok((pages, entries)) => {
            tracing::info!("Loaded {} entries from {} page(s) into the library", entries.len(), pages.len());

            if let Ok(mut library) = LIBRARY.lock() {
                *library = Some((pages, entries));
            }
        }

        Err(error) => tracing::error!("Could not load library: {error}"),
    }
}

pub fn snapshot() -> (Vec<Page>, Vec<Entry>) {
    LIBRARY
        .lock()
        .ok()
        .and_then(|library| library.clone())
        .unwrap_or_default()
}

fn load() -> Result<(Vec<Page>, Vec<Entry>), String> {
    let base = crate::data::get_data_folder()
        .ok_or("no data folder")?;

    let base = Path::new(base).join("pages");

    let dev = load_pages(&base.join(Env::Dev.dir()))?;

    if !dev.0.is_empty() {
        return Ok(dev);
    }

    load_pages(&base.join(Env::Live.dir()))
}

pub fn save_page(dir: &Path, page: &Page, entries: &[Entry]) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;

    let file = File {
        page: page.clone(),
        entries: entries.to_vec(),
    };

    let json = serde_json::to_vec_pretty(&file).map_err(|e| e.to_string())?;

    let path = dir.join(format!("{}.json", page.id));
    fs::write(&path, json).map_err(|e| e.to_string())
}

pub fn load_pages(dir: &Path) -> Result<(Vec<Page>, Vec<Entry>), String> {
    let mut pages = Vec::new();
    let mut entries = Vec::new();

    let read_dir = match fs::read_dir(dir) {
        Ok(read_dir) => read_dir,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok((pages, entries)),
        Err(err) => return Err(err.to_string()),
    };

    for entry in read_dir {
        let path = entry.map_err(|e| e.to_string())?.path();

        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let content = fs::read(&path).map_err(|e| e.to_string())?;
        let file: File = serde_json::from_slice(&content).map_err(|e| e.to_string())?;

        pages.push(file.page);
        entries.extend(file.entries);
    }

    Ok((pages, entries))
}