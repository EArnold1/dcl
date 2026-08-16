use dirs;
use std::{fs, path::PathBuf};

use crate::error::DevCloneError;

const APP_NAME: &str = "dcl";
const CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Clone)]
pub struct Paths {
    pub config_dir: PathBuf,
    pub config_file: PathBuf,
}

impl Paths {
    fn new() -> Result<Self, DevCloneError> {
        let config_dir = dirs::config_dir()
            .ok_or(DevCloneError::ConfigDirNotFound)?
            .join(APP_NAME);

        let config_file = config_dir.join(CONFIG_FILE);

        Ok(Self {
            config_dir,
            config_file,
        })
    }

    pub fn init() -> Result<Self, DevCloneError> {
        let paths = Self::new()?;
        paths.ensure_config()?;

        Ok(paths)
    }

    fn ensure_config(&self) -> Result<(), DevCloneError> {
        std::fs::create_dir_all(&self.config_dir)?;

        if !self.config_file.exists() {
            std::fs::File::create(&self.config_file)?;

            let default_config = r#"
                [symlinks]
                paths = [
                    'node_modules',
                    '.pnpm-store',
                ]

                [copies]
                paths = [
                    '.env',
                    '.env.local',
                ]

                [ignore]
                paths = [
                    'dist',
                    '.cache',
                    'coverage',
                ]
            "#;

            fs::write(&self.config_file, default_config)?;
        }

        Ok(())
    }

    pub fn config_file(&self) -> &PathBuf {
        &self.config_file
    }
}
