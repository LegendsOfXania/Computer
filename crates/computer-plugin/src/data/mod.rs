use std::{path::PathBuf, sync::OnceLock};

static DATA_FOLDER: OnceLock<PathBuf> = OnceLock::new();

pub fn init_data_folder(path: String) {
    let _ = DATA_FOLDER.set(PathBuf::from(path));
}