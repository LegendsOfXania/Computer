use std::{collections::HashMap, fs, path::Path};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{error, warn};

#[derive(Serialize, Deserialize)]
pub struct File {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub priority: i32,
    pub version: String,
    pub entries: Vec<Value>,
}

#[derive(Serialize, Deserialize)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub children: Vec<Child>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Child {
    Folder(Folder),
    File(File),
}

type Index = HashMap<String, String>;

fn load_index(base: &Path) -> Index {
    let path = base.join("index.json");
    if !path.exists() {
        return HashMap::new();
    }
    let content = fs::read_to_string(&path).unwrap_or_default();
    serde_json::from_str(&content).unwrap_or_default()
}

fn save_index(base: &Path, index: &Index) {
    let path = base.join("index.json");
    if let Ok(content) = serde_json::to_string_pretty(index) {
        if let Err(err) = fs::write(&path, content) {
            warn!(reason = %err, "Failed to save folder index");
        }
    }
}

pub fn load_tree(base: &Path) -> Vec<Folder> {
    if !base.exists() {
        if let Err(err) = fs::create_dir_all(base) {
            error!(reason = %err, "Failed to create files directory");
        }
        return Vec::new();
    }

    let mut index = load_index(base);
    let mut changed = false;
    let folders = read_folders(base, base, &mut index, &mut changed);

    if changed {
        save_index(base, &index);
    }

    folders
}

fn read_folders(base: &Path, path: &Path, index: &mut Index, changed: &mut bool) -> Vec<Folder> {
    let mut folders = Vec::new();

    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(err) => {
            error!(reason = %err, path = %path.display(), "Failed to read directory");
            return folders;
        }
    };

    for entry in entries.flatten() {
        let entry_path = entry.path();
        if entry_path.is_dir() {
            let name = entry_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let rel = entry_path
                .strip_prefix(base)
                .unwrap_or(&entry_path)
                .to_string_lossy()
                .replace('\\', "/");

            let id = index
                .entry(rel)
                .or_insert_with(|| {
                    *changed = true;
                    format!("folder_{}", uuid::Uuid::new_v4().simple())
                })
                .clone();

            let children = read_children(base, &entry_path, index, changed);
            folders.push(Folder { id, name, children });
        }
    }

    folders.sort_by(|a, b| a.name.cmp(&b.name));
    folders
}

fn read_children(base: &Path, path: &Path, index: &mut Index, changed: &mut bool) -> Vec<Child> {
    let mut children = Vec::new();

    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(err) => {
            error!(reason = %err, path = %path.display(), "Failed to read directory");
            return children;
        }
    };

    for entry in entries.flatten() {
        let entry_path = entry.path();

        if entry_path.is_dir() {
            let name = entry_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let rel = entry_path
                .strip_prefix(base)
                .unwrap_or(&entry_path)
                .to_string_lossy()
                .replace('\\', "/");

            let id = index
                .entry(rel)
                .or_insert_with(|| {
                    *changed = true;
                    format!("folder_{}", uuid::Uuid::new_v4().simple())
                })
                .clone();

            let set_children = read_children(base, &entry_path, index, changed);
            children.push(Child::Folder(Folder {
                id,
                name,
                children: set_children,
            }));
        } else if entry_path.extension().and_then(|e| e.to_str()) == Some("json") {
            match read_file(&entry_path) {
                Some(file) => children.push(Child::File(file)),
                None => warn!(path = %entry_path.display(), "Skipping invalid file"),
            }
        }
    }

    children.sort_by(|a, b| child_name(a).cmp(child_name(b)));
    children
}

fn read_file(path: &Path) -> Option<File> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(err) => {
            error!(reason = %err, path = %path.display(), "Failed to read file");
            return None;
        }
    };

    let mut value: Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(err) => {
            error!(reason = %err, path = %path.display(), "Failed to parse file");
            return None;
        }
    };

    if value.get("id").is_none() {
        let id = format!("file_{}", uuid::Uuid::new_v4().simple());
        value["id"] = Value::String(id);

        if let Ok(updated) = serde_json::to_string_pretty(&value) {
            if let Err(err) = fs::write(path, updated) {
                warn!(reason = %err, "Failed to write generated id to file");
            }
        }
    }

    serde_json::from_value(value).ok()
}

fn child_name(child: &Child) -> &str {
    match child {
        Child::Folder(folder) => &folder.name,
        Child::File(file) => &file.name,
    }
}
