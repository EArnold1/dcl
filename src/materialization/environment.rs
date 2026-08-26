use std::{
    fs,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
};

use crate::{config::loader::Config, error::DevCloneError};
use globset::{Glob, GlobSet, GlobSetBuilder};
use rayon::prelude::*;

// The EnvironmentMaterializer happens after the project has been materialized.
// We discover candidates from config symlink/copy patterns and ignore patterns in the ignore section.

#[derive(Debug)]
pub struct EnvironmentMaterializer<'a> {
    config: &'a Config,
}

impl<'a> EnvironmentMaterializer<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub fn materialize(&self, source: &Path, destination: &Path) -> Result<(), DevCloneError> {
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
        let ignore_set =
            self.build_glob_set(&self.config.ignore.paths.iter().cloned().collect::<Vec<_>>())?;

        let mut materialization_paths = Vec::new();
        self.collect_materialization_paths(
            source,
            source,
            &symlink_set,
            &copy_set,
            &ignore_set,
            &mut materialization_paths,
        )?;

        materialization_paths.into_par_iter().try_for_each(|path| {
            self.materialize_path(
                &path,
                source,
                destination,
                &symlink_set,
                &copy_set,
                &ignore_set,
            )
        })?;

        Ok(())
    }

    fn collect_materialization_paths(
        &self,
        current: &Path,
        source: &Path,
        symlink_set: &GlobSet,
        copy_set: &GlobSet,
        ignore_set: &GlobSet,
        materialization_paths: &mut Vec<PathBuf>,
    ) -> Result<(), DevCloneError> {
        // TODO: explore using WalkDir instead of recursive function
        for entry in fs::read_dir(current)? {
            let entry = entry?;
            let path = entry.path();

            if entry.file_name() == ".git" || self.matches_glob_set(&path, source, ignore_set) {
                continue;
            }

            if self.should_materialize(&path, source, symlink_set, copy_set, ignore_set) {
                materialization_paths.push(path.clone());

                if path.is_dir() {
                    continue;
                }
            }

            if path.is_dir() {
                self.collect_materialization_paths(
                    &path,
                    source,
                    symlink_set,
                    copy_set,
                    ignore_set,
                    materialization_paths,
                )?;
            }
        }

        Ok(())
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
        let relative_path = path.strip_prefix(source).unwrap_or(path);

        let candidate_paths = relative_path_ancestors(relative_path);
        candidate_paths
            .iter()
            .any(|candidate| !glob_set.matches(&*candidate.to_string_lossy()).is_empty())
    }

    fn should_materialize(
        &self,
        path: &Path,
        source: &Path,
        symlink_set: &GlobSet,
        copy_set: &GlobSet,
        ignore_set: &GlobSet,
    ) -> bool {
        if self.matches_glob_set(path, source, ignore_set) {
            return false;
        }

        self.matches_glob_set(path, source, symlink_set)
            || self.matches_glob_set(path, source, copy_set)
    }

    fn materialize_path(
        &self,
        path: &Path,
        source: &Path,
        destination: &Path,
        symlink_set: &GlobSet,
        copy_set: &GlobSet,
        ignore_set: &GlobSet,
    ) -> Result<(), DevCloneError> {
        if self.matches_glob_set(path, source, ignore_set) {
            return Ok(());
        }

        let relative_path = path.strip_prefix(source).unwrap_or(path).to_string_lossy();

        let destination_path = destination.join(&*relative_path);
        if let Some(parent) = destination_path.parent() {
            fs::create_dir_all(parent)?;
        }

        if self.matches_glob_set(path, source, symlink_set) {
            symlink(path, &destination_path)?;
        } else if self.matches_glob_set(path, source, copy_set) {
            copy(path, &destination_path)?;
        }

        Ok(())
    }
}

fn relative_path_ancestors(path: &Path) -> Vec<PathBuf> {
    let mut candidates = vec![path.to_path_buf()];
    let mut current = path;

    while let Some(parent) = current.parent() {
        if parent != current {
            candidates.push(parent.to_path_buf());
        }
        current = parent;
    }

    candidates
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignored_paths_take_precedence_over_include_patterns() {
        let config = Config {
            symlinks: crate::config::loader::PathConfig {
                paths: ["packages/**".to_string()].into_iter().collect(),
            },
            copies: crate::config::loader::PathConfig {
                paths: ["**/.env".to_string()].into_iter().collect(),
            },
            ignore: crate::config::loader::PathConfig {
                paths: ["packages/images/node_modules".to_string()]
                    .into_iter()
                    .collect(),
            },
        };

        let materializer = EnvironmentMaterializer::new(&config);
        let source = Path::new("/tmp/project");
        let symlink_set = materializer
            .build_glob_set(&config.symlinks.paths.iter().cloned().collect::<Vec<_>>())
            .unwrap();
        let copy_set = materializer
            .build_glob_set(&config.copies.paths.iter().cloned().collect::<Vec<_>>())
            .unwrap();
        let ignore_set = materializer
            .build_glob_set(&config.ignore.paths.iter().cloned().collect::<Vec<_>>())
            .unwrap();

        assert!(materializer.should_materialize(
            Path::new("/tmp/project/packages/app/src/main.rs"),
            source,
            &symlink_set,
            &copy_set,
            &ignore_set,
        ));
        assert!(!materializer.should_materialize(
            Path::new("/tmp/project/packages/images/node_modules/react/index.js"),
            source,
            &symlink_set,
            &copy_set,
            &ignore_set,
        ));
    }
}
