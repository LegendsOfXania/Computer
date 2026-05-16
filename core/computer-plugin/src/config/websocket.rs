use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct WebSocketConfig {
    pub port: u16,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self { port: 9090 }
    }
}
