pub mod instance;

use crate::{config::paths::Paths, error::DevCloneError};
pub use instance::{Instance, InstanceStatus};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Registry {
    #[serde(default)]
    pub instances: Vec<Instance>,
}

impl Registry {
    pub fn load() -> Result<Self, DevCloneError> {
        let paths = Paths::init()?;
        let registry_path = paths.registry_file();

        if !registry_path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(registry_path)?;
        toml::from_str(&content).map_err(|e| DevCloneError::RegistryParse(e.to_string()))
    }

    pub fn save(&self) -> Result<(), DevCloneError> {
        let paths = Paths::init()?;
        let registry_path = paths.registry_file();
        let serialized = toml::to_string_pretty(self)
            .map_err(|e| DevCloneError::RegistryWrite(e.to_string()))?;

        // Why write to a temporary file and then rename? This is a common pattern to avoid data loss. If the program crashes while writing, the original file remains intact. Only after successfully writing to the temporary file do we rename it to the target file name, effectively replacing it.
        let tmp_path = registry_path.with_extension("tmp");
        fs::write(&tmp_path, serialized)?;
        fs::rename(&tmp_path, registry_path)?;

        Ok(())
    }

    pub fn add(&mut self, instance: Instance) {
        self.instances.push(instance);
    }

    pub fn find_mut_by_destination(&mut self, destination: &Path) -> Option<&mut Instance> {
        self.instances
            .iter_mut()
            .find(|i| i.destination == destination)
    }

    pub fn remove(&mut self, destination: &Path) {
        self.instances.retain(|i| i.destination != destination);
    }

    pub fn resolve(&self, target: &str) -> Result<&Instance, DevCloneError> {
        if target.contains(std::path::MAIN_SEPARATOR) {
            let target_path = std::path::PathBuf::from(target);
            return self
                .instances
                .iter()
                .find(|i| i.destination == target_path)
                .ok_or_else(|| DevCloneError::InstanceNotFound(target.to_string()));
        }

        let matches: Vec<&Instance> = self.instances.iter().filter(|i| i.name == target).collect();
        match matches.as_slice() {
            [] => Err(DevCloneError::InstanceNotFound(target.to_string())),
            [only] => Ok(only),
            many => Err(DevCloneError::AmbiguousTarget {
                target: target.to_string(),
                candidates: many
                    .iter()
                    .map(|i| i.destination.display().to_string())
                    .collect(),
            }),
        }
    }
}
