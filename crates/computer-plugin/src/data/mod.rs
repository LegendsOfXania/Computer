use std::sync::OnceLock;

static DATA_FOLDER: OnceLock<String> = OnceLock::new();

pub fn init_data_folder(path: String) {
    let _ = DATA_FOLDER.set(path);
}

pub fn get_data_folder() -> Option<&'static str> {
    DATA_FOLDER.get().map(String::as_str)
}