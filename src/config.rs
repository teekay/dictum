use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::error::{DictumError, Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_prefix")]
    pub prefix: String,
    #[serde(default)]
    pub default_author: Option<String>,
    #[serde(default = "default_format")]
    pub default_format: String,
}

fn default_prefix() -> String {
    "d".to_string()
}

fn default_format() -> String {
    "auto".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            prefix: default_prefix(),
            default_author: None,
            default_format: default_format(),
        }
    }
}

impl Config {
    pub fn load(dictum_dir: &Path) -> Result<Self> {
        let config_path = dictum_dir.join("config.toml");
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .map_err(|e| DictumError::Config(e.to_string()))?;
            toml::from_str(&content).map_err(|e| DictumError::Config(e.to_string()))
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self, dictum_dir: &Path) -> Result<()> {
        let config_path = dictum_dir.join("config.toml");
        let content =
            toml::to_string_pretty(self).map_err(|e| DictumError::Config(e.to_string()))?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }
}
