use std::{fs, path::PathBuf};

use crate::error::DevCloneError;

const APP_NAME: &str = "devclone";
const CONFIG_FILE: &str = "config.toml";
const REGISTRY_FILE: &str = "registry.toml";
const DEFAULT_CONFIG: &str = r#"
[symlinks]
paths = [
    # JavaScript / TypeScript dependencies
    '**/node_modules',

    # pnpm
    '**/.pnpm-store',

    # Yarn
    '**/.yarn/cache',

    # Bun
    '**/.bun',

    # Python dependencies when used alongside JS/TS
    '**/.venv',
    '**/venv',
]

[copies]
paths = [
    # Environment configuration
    '**/.env',
    '**/.env.local',
    '**/.env.development',
    '**/.env.development.local',
    '**/.env.test',
    '**/.env.test.local',
    '**/.env.production',
    '**/.env.production.local',

    # npm configuration that can affect dependency resolution
    '**/.npmrc',

    # Yarn configuration
    '**/.yarnrc',
    '**/.yarnrc.yml',

    # Bun configuration
    '**/bunfig.toml',

    # Rust local Cargo configuration
    '**/.cargo/config.toml',
    '**/.cargo/config',
]

[ignore]
paths = [
    # Git
    '**/.git',

    # Build output
    '**/target',
    '**/dist',
    '**/build',
    '**/out',

    # Framework build/cache directories
    '**/.next',
    '**/.nuxt',
    '**/.angular',
    '**/.turbo',
    '**/.nx',
    '**/.vite',
    '**/.cache',

    # Test / tooling caches
    '**/.pytest_cache',
    '**/.jest-cache',
    '**/.vitest',

    # Logs
    '**/*.log',

    # OS files
    '**/.DS_Store',
    '**/Thumbs.db',
]
"#;

#[derive(Debug, Clone)]
pub struct Paths {
    pub config_dir: PathBuf,
    pub config_file: PathBuf,
    pub data_dir: PathBuf,
    pub registry_file: PathBuf,
}

impl Paths {
    fn new() -> Result<Self, DevCloneError> {
        let config_dir = dirs::config_dir()
            .ok_or(DevCloneError::InvalidPath(PathBuf::from("config dir")))?
            .join(APP_NAME);

        let config_file = config_dir.join(CONFIG_FILE);

        let data_dir = dirs::data_dir()
            .ok_or(DevCloneError::InvalidPath(PathBuf::from("data dir")))?
            .join(APP_NAME);

        let registry_file = data_dir.join(REGISTRY_FILE);

        Ok(Self {
            config_dir,
            config_file,
            data_dir,
            registry_file,
        })
    }

    pub fn init() -> Result<Self, DevCloneError> {
        let paths = Self::new()?;
        paths.ensure_config()?;
        paths.ensure_data_dir()?;

        Ok(paths)
    }

    fn ensure_config(&self) -> Result<(), DevCloneError> {
        fs::create_dir_all(&self.config_dir)?;

        if !self.config_file.exists() {
            fs::write(&self.config_file, DEFAULT_CONFIG)?;
        }

        Ok(())
    }

    fn ensure_data_dir(&self) -> Result<(), DevCloneError> {
        fs::create_dir_all(&self.data_dir)?;
        Ok(())
    }

    pub fn config_file(&self) -> &PathBuf {
        &self.config_file
    }

    pub fn registry_file(&self) -> &PathBuf {
        &self.registry_file
    }
}
