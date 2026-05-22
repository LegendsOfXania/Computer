use serde::{Deserialize, Serialize};
use std::{fs, path::Path, sync::OnceLock};
use tracing::{error, warn};

static CONFIG: OnceLock<ComputerConfig> = OnceLock::new();

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct ComputerConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
}

impl Default for ComputerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            host: "127.0.0.1".to_string(),
            port: 8080,
        }
    }
}

impl ComputerConfig {
    pub fn init(plugin_dir: &Path) {
        let config = Self::load(plugin_dir);
        CONFIG
            .set(config)
            .expect("ComputerConfig already initialized");
    }

    pub fn get() -> &'static ComputerConfig {
        CONFIG.get().expect("ComputerConfig not initialized")
    }

    pub fn panel_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn panel_active(&self) -> bool {
        self.enabled
    }

    fn load(plugin_dir: &Path) -> Self {
        let path = plugin_dir.join("config.toml");

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => {
                let default = Self::default();
                if let Err(e) = fs::write(&path, toml::to_string(&default).unwrap()) {
                    warn!(reason = %e, "Could not write default config.toml");
                }
                return default;
            }
        };

        match toml::from_str::<Self>(&content) {
            Ok(cfg) => cfg,
            Err(e) => {
                error!(reason = %e, "Could not parse config.toml, using defaults");
                Self::default()
            }
        }
    }
}
