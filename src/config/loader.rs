use toml;

use crate::{config::paths::Paths, error::DevCloneError};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub symlinks: PathConfig,
    pub copies: PathConfig,
    pub ignore: PathConfig,
}

#[derive(Debug, Deserialize)]
pub struct PathConfig {
    pub paths: Vec<String>,
}

impl Config {
    pub fn load_from_file() -> Result<Self, DevCloneError> {
        let paths = Paths::init()?;
        let config_path = paths.config_file();
        let config_content = std::fs::read_to_string(config_path)?;
        let user_config: Config =
            toml::from_str(&config_content).expect("config file should be a valid toml file");

        // TODO: Validate the config structure
        // Make sure a path doesn't appear in more than one category.
        Ok(user_config)
    }
}
