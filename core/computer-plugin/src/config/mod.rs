pub mod networking;

use std::{fs, path::Path};

use networking::NetworkingConfig;
use serde::{Deserialize, Serialize};
use toml::Value;
use tracing::{error, warn};

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct ComputerConfig {
    pub networking: NetworkingConfig,
}

impl ComputerConfig {
    #[must_use]
    pub fn load(plugin_dir: &Path) -> Self {
        let path = plugin_dir.join("config.toml");

        let config = if path.exists() {
            let content = match fs::read_to_string(&path) {
                Ok(content) => content,
                Err(err) => {
                    error!(reason = %err, "Could not read config.toml, using default values");
                    return Self::default();
                }
            };

            let parsed: Value = match toml::from_str(&content) {
                Ok(parsed) => parsed,
                Err(err) => {
                    error!(reason = %err, "Could not parse config.toml, using default values");
                    return Self::default();
                }
            };

            let (merged, changed) = Self::merge_with_defaults(parsed);

            if changed {
                warn!("Missing fields have been filled with default values");
                if let Err(err) = fs::write(&path, toml::to_string(&merged).unwrap()) {
                    warn!(reason = %err, "Could not save config.toml after merge");
                }
            }

            merged
        } else {
            let default = Self::default();
            if let Err(err) = fs::write(&path, toml::to_string(&default).unwrap()) {
                warn!(reason = %err, "Could not write config.toml by default");
            }
            default
        };

        config
    }

    fn merge_with_defaults(parsed: Value) -> (Self, bool) {
        let default_value =
            Value::try_from(Self::default()).expect("Failed to serialize default config");

        let (merged_value, changed) = Self::merge_toml_values(default_value, parsed);

        let config = merged_value
            .try_into()
            .expect("Failed to deserialize after merge");

        (config, changed)
    }

    fn merge_toml_values(base: Value, overlay: Value) -> (Value, bool) {
        match (base, overlay) {
            (Value::Table(mut base_table), Value::Table(overlay_table)) => {
                let mut changed = base_table.keys().any(|k| !overlay_table.contains_key(k));

                for (key, overlay_value) in overlay_table {
                    if let Some(base_value) = base_table.get(&key).cloned() {
                        let (merged, value_changed) =
                            Self::merge_toml_values(base_value, overlay_value);
                        base_table.insert(key, merged);
                        if value_changed {
                            changed = true;
                        }
                    } else {
                        base_table.insert(key, overlay_value);
                    }
                }

                (Value::Table(base_table), changed)
            }
            (_, overlay) => (overlay, false),
        }
    }
}
