use crate::error::DevCloneError;
use crate::info;
use crate::registry::Registry;
use crate::warn;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub fn remove(target: String, yes: bool) -> Result<(), DevCloneError> {
    let mut registry = Registry::load()?;
    let instance = registry.resolve(&target)?.clone();

    if !yes && !confirm(&instance.name, &instance.destination) {
        info!("Aborted.");
        return Ok(());
    }

    if instance.destination.exists() {
        fs::remove_dir_all(&instance.destination)?;
    } else {
        warn!(
            "Instance directory {:?} does not exist; removing registry entry only.",
            instance.destination
        );
    }

    registry.remove(&instance.destination);
    registry.save()?;
    info!("Removed instance '{}'.", instance.name);
    Ok(())
}

/// Prompts the user for confirmation to remove an instance.
fn confirm(name: &str, path: &Path) -> bool {
    print!("Remove instance '{name}' at {}? [y/N] ", path.display());
    io::stdout().flush().ok();
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return false;
    }
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}
