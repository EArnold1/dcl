use crate::error::DevCloneError;
use std::path::Path;

#[cfg(unix)]
pub fn create_symlink(target: &Path, link: &Path) -> Result<(), DevCloneError> {
    std::os::unix::fs::symlink(target, link).map_err(DevCloneError::Io)
}

#[cfg(windows)]
pub fn create_symlink(target: &Path, link: &Path) -> Result<(), DevCloneError> {
    if target.is_dir() {
        std::os::windows::fs::symlink_dir(target, link)
    } else {
        std::os::windows::fs::symlink_file(target, link)
    }
    .map_err(|e| DevCloneError::SymlinkFailed {
        target: target.to_path_buf(),
        link: link.to_path_buf(),
        reason: e.to_string(),
    })
}
