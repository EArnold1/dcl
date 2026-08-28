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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_full_config() {
        let toml_str = r#"
            [symlinks]
            paths = ["**/node_modules"]

            [copies]
            paths = ["**/.env"]

            [ignore]
            paths = ["**/.git"]
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();

        assert_eq!(
            config.symlinks.paths,
            HashSet::from(["**/node_modules".to_string()])
        );
        assert_eq!(config.copies.paths, HashSet::from(["**/.env".to_string()]));
        assert_eq!(config.ignore.paths, HashSet::from(["**/.git".to_string()]));
    }

    #[test]
    fn missing_sections_default_to_empty() {
        let config: Config = toml::from_str("").unwrap();

        assert!(config.symlinks.paths.is_empty());
        assert!(config.copies.paths.is_empty());
        assert!(config.ignore.paths.is_empty());
    }

    #[test]
    fn malformed_toml_returns_config_parse_error() {
        let result: Result<Config, DevCloneError> = toml::from_str("not = [valid")
            .map_err(|err| DevCloneError::ConfigParse(err.to_string()));

        assert!(matches!(result, Err(DevCloneError::ConfigParse(_))));
    }
}
