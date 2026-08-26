use std::env;
use std::path::PathBuf;

use crate::error::DevCloneError;

#[derive(Debug)]
pub struct ProjectIdentity {
    pub name: String,
    pub root_path: PathBuf,
}

impl ProjectIdentity {
    pub fn discover() -> Result<Self, DevCloneError> {
        let root_path = env::current_dir()?;

        let name = root_path
            .file_name()
            .ok_or(DevCloneError::ProjectNameNotFound)?
            .to_string_lossy()
            .into_owned();

        Ok(Self { name, root_path })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn discover_returns_current_dir_name_and_path() {
        let tmp = TempDir::new().unwrap();
        let original_dir = env::current_dir().unwrap();

        env::set_current_dir(tmp.path()).unwrap();
        let result = ProjectIdentity::discover();
        env::set_current_dir(original_dir).unwrap();

        let identity = result.unwrap();
        assert_eq!(identity.root_path.file_name(), tmp.path().file_name());
        assert_eq!(
            identity.name,
            tmp.path().file_name().unwrap().to_string_lossy()
        );
    }
}
