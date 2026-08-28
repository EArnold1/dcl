use std::{fs, path::PathBuf};

use crate::error::DevCloneError;

const APP_NAME: &str = "devclone";
const CONFIG_FILE: &str = "config.toml";
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
}

impl Paths {
    fn new() -> Result<Self, DevCloneError> {
        let config_dir = dirs::config_dir()
            .ok_or(DevCloneError::InvalidPath(PathBuf::from("config dir")))?
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
        fs::create_dir_all(&self.config_dir)?;

        if !self.config_file.exists() {
            fs::write(&self.config_file, DEFAULT_CONFIG)?;
        }

        Ok(())
    }

    pub fn config_file(&self) -> &PathBuf {
        &self.config_file
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::loader::Config;
    use tempfile::TempDir;

    #[test]
    fn new_joins_app_name_and_config_file() {
        let paths = Paths::new().unwrap();

        assert_eq!(paths.config_dir.file_name().unwrap(), APP_NAME);
        assert_eq!(paths.config_file, paths.config_dir.join(CONFIG_FILE));
    }

    #[test]
    fn ensure_config_creates_dir_and_writes_default_when_missing() {
        let tmp = TempDir::new().unwrap();
        let config_dir = tmp.path().join(APP_NAME);
        let paths = Paths {
            config_dir: config_dir.clone(),
            config_file: config_dir.join(CONFIG_FILE),
        };

        paths.ensure_config().unwrap();

        assert!(paths.config_dir.is_dir());
        assert_eq!(
            fs::read_to_string(&paths.config_file).unwrap(),
            DEFAULT_CONFIG
        );
    }

    #[test]
    fn ensure_config_does_not_overwrite_existing_file() {
        let tmp = TempDir::new().unwrap();
        let config_dir = tmp.path().join(APP_NAME);
        fs::create_dir_all(&config_dir).unwrap();
        let config_file = config_dir.join(CONFIG_FILE);
        fs::write(&config_file, "custom = true").unwrap();

        let paths = Paths {
            config_dir,
            config_file: config_file.clone(),
        };
        paths.ensure_config().unwrap();

        assert_eq!(fs::read_to_string(&config_file).unwrap(), "custom = true");
    }

    #[test]
    fn default_config_parses_as_valid_config() {
        let config: Config = toml::from_str(DEFAULT_CONFIG).unwrap();

        assert!(config.symlinks.paths.contains("**/node_modules"));
        assert!(config.copies.paths.contains("**/.env"));
        assert!(config.ignore.paths.contains("**/.git"));
    }
}
