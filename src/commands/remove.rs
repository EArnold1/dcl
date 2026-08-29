use crate::cleanup::cleanup;
use crate::error::DevCloneError;
use crate::info;
use crate::registry::Registry;
use std::io::{self, Write};
use std::path::Path;

pub fn remove(target: String, yes: bool) -> Result<(), DevCloneError> {
    if !yes {
        let registry = Registry::load()?;
        let instance = registry.resolve(&target)?; // clone to end the immutable borrow
        if !confirm(&instance.name, &instance.destination) {
            info!("Aborted.");
            return Ok(());
        }
    }
    cleanup(&target)
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
