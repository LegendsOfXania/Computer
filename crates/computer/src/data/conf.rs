use std::{fs, net::Ipv4Addr, path::Path, sync::OnceLock};

use serde::{Deserialize, Serialize};

use crate::data;

static CONFIG: OnceLock<ComputerConfig> = OnceLock::new();

#[derive(Deserialize, Serialize, Default)]
pub struct ComputerConfig {
    pub panel: PanelConfig,
}

#[derive(Deserialize, Serialize)]
pub struct PanelConfig {
    pub enabled: bool,
    pub ip: String,
    pub port: u16,
}

impl Default for PanelConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            ip: Ipv4Addr::LOCALHOST.to_string(),
            port: 8080,
        }
    }
}

fn init_conf() -> ComputerConfig {
    let Some(data_folder) = data::get_data_folder() else {
        return ComputerConfig::default();
    };

    let path = Path::new(data_folder).join("config.toml");

    let config = match fs::read_to_string(&path) {
        Ok(content) => toml::from_str(&content).unwrap_or_else(|error| {
            tracing::error!(
                "Invalid {}, using the default configuration: {error}",
                path.display()
            );

            ComputerConfig::default()
        }),

        Err(_) => ComputerConfig::default(),
    };

    if !path.exists() {
        if let Ok(content) = toml::to_string_pretty(&config) {
            let _ = fs::write(&path, content);
        }
    }

    config
}

pub fn get_conf() -> &'static ComputerConfig {
    CONFIG.get_or_init(init_conf)
}