use std::path::Path;

use serde::Deserialize;
use waki::Client;

use crate::data;

const PANEL_ARCHIVE: &str = "panel.tar.gz";
const INSTALLED_VERSION: &str = ".installed-version";

pub fn ensure_panel_up_to_date() {
    let Some(data_folder) = data::get_data_folder() else {
        return;
    };

    let panel = Path::new(data_folder).join("assets/panel");

    tracing::info!("Checking for panel update...");

    match update(&panel) {
        Ok(Some(version)) => tracing::info!("Panel updated to {version}"),
        Ok(None) => tracing::info!("Panel is up to date"),
        Err(error) => tracing::error!("Could not update panel: {error}"),
    }
}

fn update(panel: &Path) -> Result<Option<String>, String> {
    let installed = std::fs::read_to_string(panel.join(INSTALLED_VERSION))
        .ok()
        .map(|version| version.trim().to_owned());

    let latest = latest_version()?;

    if installed.as_deref() == Some(&latest) && panel.join("index.html").is_file() {
        return Ok(None);
    }

    let archive = download(&format!(
        "https://github.com/LegendsOfXania/Computer/releases/download/{latest}/{PANEL_ARCHIVE}"
    ))?;

    install(panel, &archive)?;
    std::fs::write(panel.join(INSTALLED_VERSION), &latest)
        .map_err(|e| e.to_string())?;

    Ok(Some(latest))
}

fn latest_version() -> Result<String, String> {
    let body = download(
        "https://api.github.com/repos/LegendsOfXania/Computer/releases/latest",
    )?;

    serde_json::from_slice::<Release>(&body)
        .map(|release| release.tag_name)
        .map_err(|e| e.to_string())
}

fn download(url: &str) -> Result<Vec<u8>, String> {
    let response = Client::new()
        .get(url)
        .header(
            "User-Agent",
            concat!("computer-plugin/", env!("CARGO_PKG_VERSION")),
        )
        .send()
        .map_err(|e| e.to_string())?;

    if matches!(response.status_code(), 301 | 302 | 303 | 307 | 308) {
        let location = response
            .header("Location")
            .ok_or("redirect without location")?;

        return download(location.to_str().map_err(|_| "invalid redirect")?);
    }

    if response.status_code() != 200 {
        return Err(format!("HTTP {}", response.status_code()));
    }

    response.body().map_err(|e| e.to_string())
}

fn install(panel: &Path, archive: &[u8]) -> Result<(), String> {
    let parent = panel.parent().ok_or("invalid panel path")?;
    let temp = parent.join("panel_tmp");

    if temp.exists() {
        std::fs::remove_dir_all(&temp).map_err(|e| e.to_string())?;
    }

    std::fs::create_dir_all(&temp).map_err(|e| e.to_string())?;

    let decoder = flate2::read::GzDecoder::new(archive);
    let mut archive = tar::Archive::new(decoder);

    // archive.unpack does not work

    for entry in archive.entries().map_err(|e| e.to_string())? {
        let mut entry = entry.map_err(|e| e.to_string())?;
        let path = temp.join(entry.path().map_err(|e| e.to_string())?);

        if entry.header().entry_type().is_dir() {
            std::fs::create_dir_all(path).map_err(|e| e.to_string())?;
            continue;
        }

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        std::io::copy(
            &mut entry,
            &mut std::fs::File::create(path).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
    }

    if panel.exists() {
        std::fs::remove_dir_all(panel).map_err(|e| e.to_string())?;
    }

    std::fs::rename(temp, panel).map_err(|e| e.to_string())
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
}