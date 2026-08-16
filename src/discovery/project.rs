use crate::error::DevCloneError;
use std::path::PathBuf;

// What would be the properties for a project identity?

#[derive(Debug)]
pub struct ProjectIdentity {
    name: String,       // name of the instance
    root_path: PathBuf, // project root
}

fn discover() -> Result<PathBuf, DevCloneError> {
    Ok(std::env::current_dir()?)
}

impl ProjectIdentity {
    pub fn new() -> Result<Self, DevCloneError> {
        let root = discover()?;
        let name = &root
            .file_name()
            .ok_or(DevCloneError::ProjectNameNotFound)?
            .to_string_lossy()
            .to_string();

        Ok(Self {
            name: name.to_owned(),
            root_path: discover()?,
        })
    }
}
