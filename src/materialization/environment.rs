use std::{
    collections::HashSet,
    fs,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
};

use crate::{config::loader::Config, error::DevCloneError};

// The EnvironmentMaterializer happens after the project has been materialized, which is done by git.
// This means we can just rely on the .gitignore file to determine the ignored files, and then apply the symlinks and copies as specified in the config.
// This might change if we decide to not use git for materialization.
// If git becomes optional, then we can do environment materialization before project materialization and not entirely rely on the .gitignore file.

#[derive(Debug)]
pub struct EnvironmentMaterializer<'a> {
    config: &'a Config,
}

impl<'a> EnvironmentMaterializer<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub fn materialize(&self, source: &Path, destination: &Path) -> Result<(), DevCloneError> {
        let ignored = self.discover_ignored(source)?;

        // TODO: Use threads to process multiple paths concurrently
        for path in ignored {
            self.materialize_path(&path, source, destination)?;
        }

        Ok(())
    }

    fn discover_ignored(&self, source: &Path) -> Result<Vec<PathBuf>, DevCloneError> {
        let contents = fs::read_to_string(source.join(".gitignore"))?;

        let ignored: HashSet<&str> = contents
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .collect();

        let paths = fs::read_dir(source)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| ignored.contains(name)) // Implement pattern matching "/*, /*/*.ext, /dir/*"
            })
            .collect();

        Ok(paths)
    }

    fn materialize_path(
        &self,
        path: &Path,
        source: &Path,
        destination: &Path,
    ) -> Result<(), DevCloneError> {
        let name = path
            .file_name()
            .ok_or_else(|| DevCloneError::InvalidPath(path.to_owned()))?;

        let name = name.to_string_lossy();

        let source_path = source.join(&*name);
        let destination_path = destination.join(&*name);

        if self.config.symlinks.paths.contains(name.as_ref()) {
            symlink(&source_path, &destination_path)?;
        } else if self.config.copies.paths.contains(name.as_ref()) {
            copy(&source_path, &destination_path)?;
        }

        Ok(())
    }
}

fn copy(source: &Path, destination: &Path) -> Result<(), DevCloneError> {
    if source.is_dir() {
        copy_dir(source, destination)
    } else {
        fs::copy(source, destination)?;
        Ok(())
    }
}

fn copy_dir(source: &Path, destination: &Path) -> Result<(), DevCloneError> {
    fs::create_dir_all(destination)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());

        if source_path.is_dir() {
            copy_dir(&source_path, &destination_path)?;
        } else {
            fs::copy(&source_path, &destination_path)?;
        }
    }

    Ok(())
}
