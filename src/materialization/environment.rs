use std::{
    fs,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
};

use crate::{config::loader::Config, error::DevCloneError};
use globset::{Glob, GlobSet, GlobSetBuilder};
use rayon::prelude::*;

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

        let symlink_set = self.build_glob_set(
            &self
                .config
                .symlinks
                .paths
                .iter()
                .cloned()
                .collect::<Vec<_>>(),
        )?;
        let copy_set =
            self.build_glob_set(&self.config.copies.paths.iter().cloned().collect::<Vec<_>>())?;

        ignored.into_par_iter().try_for_each(|path| {
            self.materialize_path(&path, source, destination, &symlink_set, &copy_set)
        })?;

        Ok(())
    }

    fn discover_ignored(&self, source: &Path) -> Result<Vec<PathBuf>, DevCloneError> {
        let contents = fs::read_to_string(source.join(".gitignore"))?;

        let patterns: Vec<String> = contents
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|s| s.to_string())
            .collect();

        let glob_set = self.build_glob_set(&patterns)?;

        let mut ignored_paths = Vec::new();

        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let path = entry.path();

            if self.matches_glob_set(&path, source, &glob_set) {
                ignored_paths.push(path);
            }
        }

        Ok(ignored_paths)
    }

    fn build_glob_set(&self, patterns: &[String]) -> Result<GlobSet, DevCloneError> {
        let mut builder = GlobSetBuilder::new();

        for pattern in patterns {
            let normalized = if let Some(stripped) = pattern.strip_prefix('/') {
                stripped.to_string()
            } else {
                pattern.clone()
            };

            builder.add(
                Glob::new(&normalized).map_err(|e| {
                    DevCloneError::InvalidGlobPattern(format!("{}: {}", pattern, e))
                })?,
            );
        }

        builder
            .build()
            .map_err(|e| DevCloneError::GlobSetBuild(e.to_string()))
    }

    fn matches_glob_set(&self, path: &Path, source: &Path, glob_set: &GlobSet) -> bool {
        let relative_path = path.strip_prefix(source).unwrap_or(path).to_string_lossy();

        !glob_set.matches(&*relative_path).is_empty()
    }

    fn materialize_path(
        &self,
        path: &Path,
        source: &Path,
        destination: &Path,
        symlink_set: &GlobSet,
        copy_set: &GlobSet,
    ) -> Result<(), DevCloneError> {
        let relative_path = path.strip_prefix(source).unwrap_or(path).to_string_lossy();

        let destination_path = destination.join(&*relative_path);

        if self.matches_glob_set(path, source, symlink_set) {
            symlink(path, &destination_path)?;
        } else if self.matches_glob_set(path, source, copy_set) {
            copy(path, &destination_path)?;
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
