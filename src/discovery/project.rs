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
            .ok_or(DevCloneError::ProjectNameNotFound)? // TODO: use a wider error type
            .to_string_lossy()
            .into_owned();

        Ok(Self { name, root_path })
    }
}
