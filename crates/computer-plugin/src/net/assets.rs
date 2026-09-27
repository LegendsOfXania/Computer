use std::{collections::HashMap, fs, path::Path, sync::Arc};

use crate::data;

pub struct Asset {
    pub content_type: &'static str,
    pub body: Arc<[u8]>,
}

pub fn load() -> Option<HashMap<String, Asset>> {
    let data_folder = data::get_data_folder()?;
    let panel_dir = Path::new(data_folder)
        .join("assets/panel")
        .canonicalize()
        .ok()?;

    let mut assets = HashMap::new();
    collect(&panel_dir, &panel_dir, &mut assets);

    if assets.is_empty() {
        tracing::warn!("Panel directory is empty, is the panel installed?");
    }

    Some(assets)
}

fn collect(root: &Path, dir: &Path, out: &mut HashMap<String, Asset>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            collect(root, &path, out);
            continue;
        }

        let (Ok(body), Ok(relative)) = (fs::read(&path), path.strip_prefix(root)) else {
            continue;
        };

        let key = relative.to_string_lossy().replace('\\', "/");
        out.insert(
            key,
            Asset {
                content_type: content_type(&path),
                body: Arc::from(body),
            },
        );
    }
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("wasm") => "application/wasm",
        Some("json") => "application/json",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    }
}