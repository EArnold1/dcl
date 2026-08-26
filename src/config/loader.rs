use std::collections::HashSet;

use serde::Deserialize;

use crate::{config::paths::Paths, error::DevCloneError};

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub symlinks: PathConfig,
    #[serde(default)]
    pub copies: PathConfig,
    #[serde(default)]
    pub ignore: PathConfig,
}

#[derive(Debug, Deserialize, Default)]
pub struct PathConfig {
    #[serde(default)]
    pub paths: HashSet<String>,
}

impl Config {
    pub fn load_from_file() -> Result<Self, DevCloneError> {
        let paths = Paths::init()?;
        let config_path = paths.config_file();
        let config_content = std::fs::read_to_string(config_path)?;

        toml::from_str(&config_content).map_err(|err| DevCloneError::ConfigParse(err.to_string()))
    }
}
