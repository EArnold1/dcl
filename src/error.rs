use std::{io, path::PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DevCloneError {
    #[error("I/O error occurred: {0}")]
    Io(#[from] io::Error),
    #[error("unsupported directory entry type: {0}")]
    UnsupportedEntry(PathBuf),
    #[error("Config dir not found")]
    ConfigDirNotFound,
    #[error("Project name not found")]
    ProjectNameNotFound,
    #[error("Invalid path")]
    InvalidPath,
    #[error("Command failed")]
    CommandFailed,
}
