use std::fs;
use std::path::Path;

use crate::error::DevCloneError;
use crate::info;
use crate::registry::Registry;
use crate::warn;

pub fn cleanup_filesystem(destination: &Path) -> Result<(), DevCloneError> {
    if destination.exists() {
        fs::remove_dir_all(destination)?;
    }
    Ok(())
}

pub fn cleanup(target: &str) -> Result<(), DevCloneError> {
    let mut registry = Registry::load()?;
    let instance = registry.resolve(target)?.clone();

    cleanup_filesystem(&instance.destination)?;

    if !instance.destination.exists() {
        registry.remove(&instance.destination);
        registry.save()?;
        info!("Removed instance '{}'.", instance.name);
    } else {
        warn!(
            "Failed to completely remove instance directory {:?}; registry entry removed.",
            instance.destination
        );
        registry.remove(&instance.destination);
        registry.save()?;
    }

    Ok(())
}
