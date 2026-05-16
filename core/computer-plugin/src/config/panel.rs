use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct PanelConfig {
    pub enabled: bool,
    pub port: u16,
}

impl Default for PanelConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: 8080,
        }
    }
}
