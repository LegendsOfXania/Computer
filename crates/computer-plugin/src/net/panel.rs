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

    if installed.as_deref() == Some(&latest)
        && panel.join("index.html").is_file()
    {
        return Ok(None);
    }

    let archive = download(&format!(
        "https://github.com/LegendsOfXania/Computer/releases/download/{latest}/{PANEL_ARCHIVE}"
    ))?;

    install(panel, &archive)?;

    std::fs::write(panel.join(INSTALLED_VERSION), &latest)
        .map_err(|error| error.to_string())?;

    Ok(Some(latest))
}

fn latest_version() -> Result<String, String> {
    let url = format!(
        "https://api.github.com/repos/LegendsOfXania/Computer/releases/latest"
    );

    let body = download(&url)?;
    let release: Release =
        serde_json::from_slice(&body).map_err(|error| error.to_string())?;

    Ok(release.tag_name)
}

fn download(url: &str) -> Result<Vec<u8>, String> {
    Client::new()
        .get(url)
        .header(
            "User-Agent",
            concat!("computer-plugin/", env!("CARGO_PKG_VERSION")),
        )
        .send()
        .map_err(|error| error.to_string())?
        .body()
        .map_err(|error| error.to_string())
}

fn install(panel: &Path, archive: &[u8]) -> Result<(), String> {
    let parent = panel.parent().ok_or("invalid panel path")?;
    let temp = parent.join("panel.tmp");

    std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;

    if temp.exists() {
        std::fs::remove_dir_all(&temp).map_err(|error| error.to_string())?;
    }

    std::fs::create_dir_all(&temp).map_err(|error| error.to_string())?;

    let decoder = flate2::read::GzDecoder::new(archive);
    let mut tar = tar::Archive::new(decoder);

    tar.unpack(&temp).map_err(|error| error.to_string())?;

    if panel.exists() {
        std::fs::remove_dir_all(panel).map_err(|error| error.to_string())?;
    }

    std::fs::rename(temp, panel).map_err(|error| error.to_string())
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
}