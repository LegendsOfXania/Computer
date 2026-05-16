use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct NetworkingConfig {
    pub address: SocketAddr,
}

impl Default for NetworkingConfig {
    fn default() -> Self {
        Self {
            address: "0.0.0.0:8080".parse().unwrap(),
        }
    }
}
