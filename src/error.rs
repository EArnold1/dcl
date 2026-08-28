use std::{io, path::PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DevCloneError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("unsupported directory entry type: {0}")]
    UnsupportedEntry(PathBuf),

    #[error("project name could not be determined")]
    ProjectNameNotFound,

    #[error("could not determine parent directory for: {0}")]
    InvalidPath(PathBuf),

    #[error("invalid glob pattern: {0}")]
    InvalidGlobPattern(String),

    #[error("failed to build glob set: {0}")]
    GlobSetBuild(String),

    #[error("failed to parse config: {0}")]
    ConfigParse(String),

    #[error("command `{command}` failed: {stderr}")]
    CommandFailed { command: String, stderr: String },

    #[error("destination already exists: {0}")]
    DestinationExists(PathBuf),

    #[error("failed to parse registry: {0}")]
    RegistryParse(String),

    #[error("failed to write registry: {0}")]
    RegistryWrite(String),

    #[error("no managed instance found matching: {0}")]
    InstanceNotFound(String),

    #[error("multiple instances match '{target}': {candidates:?}; specify the full destination path to disambiguate")]
    AmbiguousTarget { target: String, candidates: Vec<String> },
}
