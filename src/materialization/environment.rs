use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    config::loader::Config, error::DevCloneError, materialization::symlink::create_symlink,
};
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
            create_symlink(path, &destination_path)?;
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
    use crate::config::loader::PathConfig;

    use super::*;

    #[test]
    fn ignored_paths_take_precedence_over_include_patterns() {
        let config = Config {
            symlinks: PathConfig {
                paths: ["packages/**".to_string()].into_iter().collect(),
            },
            copies: PathConfig {
                paths: ["**/.env".to_string()].into_iter().collect(),
            },
            ignore: PathConfig {
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

    #[test]
    fn build_glob_set_strips_leading_slash() {
        let config = Config::default();
        let materializer = EnvironmentMaterializer::new(&config);

        let glob_set = materializer
            .build_glob_set(&["/node_modules".to_string()])
            .unwrap();

        assert!(!glob_set.matches("node_modules").is_empty());
    }

    #[test]
    fn build_glob_set_returns_error_for_invalid_pattern() {
        let config = Config::default();
        let materializer = EnvironmentMaterializer::new(&config);

        let result = materializer.build_glob_set(&["[invalid".to_string()]);

        assert!(matches!(result, Err(DevCloneError::InvalidGlobPattern(_))));
    }

    #[test]
    fn matches_glob_set_matches_via_ancestor_segment() {
        let config = Config::default();
        let materializer = EnvironmentMaterializer::new(&config);
        let ignore_set = materializer
            .build_glob_set(&["**/node_modules".to_string()])
            .unwrap();
        let source = Path::new("/tmp/project");
        let path = Path::new("/tmp/project/packages/app/node_modules/react/index.js");

        assert!(materializer.matches_glob_set(path, source, &ignore_set));
    }

    #[test]
    fn should_materialize_true_when_only_copy_set_matches() {
        let config = Config::default();
        let materializer = EnvironmentMaterializer::new(&config);
        let symlink_set = materializer.build_glob_set(&[]).unwrap();
        let copy_set = materializer
            .build_glob_set(&["**/.env".to_string()])
            .unwrap();
        let ignore_set = materializer.build_glob_set(&[]).unwrap();
        let source = Path::new("/tmp/project");

        assert!(materializer.should_materialize(
            Path::new("/tmp/project/.env"),
            source,
            &symlink_set,
            &copy_set,
            &ignore_set,
        ));
    }

    #[test]
    fn should_materialize_true_when_only_symlink_set_matches() {
        let config = Config::default();
        let materializer = EnvironmentMaterializer::new(&config);
        let symlink_set = materializer
            .build_glob_set(&["**/node_modules".to_string()])
            .unwrap();
        let copy_set = materializer.build_glob_set(&[]).unwrap();
        let ignore_set = materializer.build_glob_set(&[]).unwrap();
        let source = Path::new("/tmp/project");

        assert!(materializer.should_materialize(
            Path::new("/tmp/project/node_modules"),
            source,
            &symlink_set,
            &copy_set,
            &ignore_set,
        ));
    }

    #[test]
    fn relative_path_ancestors_includes_path_and_all_parents() {
        let ancestors = relative_path_ancestors(Path::new("a/b/c"));

        assert!(ancestors.contains(&PathBuf::from("a/b/c")));
        assert!(ancestors.contains(&PathBuf::from("a/b")));
        assert!(ancestors.contains(&PathBuf::from("a")));
    }

    #[test]
    fn collect_materialization_paths_does_not_recurse_into_matched_directories() {
        let tmp = tempfile::TempDir::new().unwrap();
        let source = tmp.path();

        fs::create_dir_all(source.join(".git")).unwrap();
        fs::write(source.join(".git/HEAD"), "ref: refs/heads/main").unwrap();

        fs::create_dir_all(source.join("node_modules/react")).unwrap();
        fs::write(source.join("node_modules/react/index.js"), "").unwrap();

        fs::write(source.join(".env"), "SECRET=1").unwrap();

        let config = Config {
            symlinks: PathConfig {
                paths: ["**/node_modules".to_string()].into_iter().collect(),
            },
            copies: PathConfig {
                paths: ["**/.env".to_string()].into_iter().collect(),
            },
            ignore: PathConfig::default(),
        };

        let materializer = EnvironmentMaterializer::new(&config);
        let symlink_set = materializer
            .build_glob_set(&config.symlinks.paths.iter().cloned().collect::<Vec<_>>())
            .unwrap();
        let copy_set = materializer
            .build_glob_set(&config.copies.paths.iter().cloned().collect::<Vec<_>>())
            .unwrap();
        let ignore_set = materializer
            .build_glob_set(&config.ignore.paths.iter().cloned().collect::<Vec<_>>())
            .unwrap();

        let mut paths = Vec::new();
        materializer
            .collect_materialization_paths(
                source,
                source,
                &symlink_set,
                &copy_set,
                &ignore_set,
                &mut paths,
            )
            .unwrap();

        assert!(!paths.iter().any(|p| p.starts_with(source.join(".git"))));
        assert!(paths.contains(&source.join("node_modules")));
        assert!(!paths.contains(&source.join("node_modules/react")));
        assert!(paths.contains(&source.join(".env")));
    }

    #[cfg(unix)]
    #[test]
    fn collect_materialization_paths_does_not_descend_into_ignored_directories() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::TempDir::new().unwrap();
        let source = tmp.path();

        let ignored_dir = source.join("target");
        let locked_dir = ignored_dir.join("locked");
        fs::create_dir_all(&locked_dir).unwrap();

        // permission-locked so any attempt to descend into it would error
        let mut perms = fs::metadata(&locked_dir).unwrap().permissions();
        perms.set_mode(0o000);
        fs::set_permissions(&locked_dir, perms).unwrap();

        let config = Config {
            symlinks: PathConfig::default(),
            copies: PathConfig::default(),
            ignore: PathConfig {
                paths: ["**/target".to_string()].into_iter().collect(),
            },
        };

        let materializer = EnvironmentMaterializer::new(&config);
        let symlink_set = materializer.build_glob_set(&[]).unwrap();
        let copy_set = materializer.build_glob_set(&[]).unwrap();
        let ignore_set = materializer
            .build_glob_set(&config.ignore.paths.iter().cloned().collect::<Vec<_>>())
            .unwrap();

        let mut paths = Vec::new();
        let result = materializer.collect_materialization_paths(
            source,
            source,
            &symlink_set,
            &copy_set,
            &ignore_set,
            &mut paths,
        );

        let mut perms = fs::metadata(&locked_dir).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&locked_dir, perms).unwrap();

        result.unwrap();
        assert!(paths.is_empty());
    }

    #[test]
    fn materialize_creates_symlinks_copies_and_skips_ignored_paths() {
        let tmp = tempfile::TempDir::new().unwrap();
        let source = tmp.path().join("source");
        let destination = tmp.path().join("destination");
        fs::create_dir_all(&source).unwrap();

        fs::create_dir_all(source.join("node_modules/react")).unwrap();
        fs::write(source.join("node_modules/react/index.js"), "react").unwrap();

        fs::write(source.join(".env"), "SECRET=1").unwrap();

        fs::create_dir_all(source.join("target")).unwrap();
        fs::write(source.join("target/output"), "binary").unwrap();

        let config = Config {
            symlinks: PathConfig {
                paths: ["**/node_modules".to_string()].into_iter().collect(),
            },
            copies: PathConfig {
                paths: ["**/.env".to_string()].into_iter().collect(),
            },
            ignore: PathConfig {
                paths: ["**/target".to_string()].into_iter().collect(),
            },
        };

        let materializer = EnvironmentMaterializer::new(&config);
        materializer.materialize(&source, &destination).unwrap();

        let symlinked_node_modules = destination.join("node_modules");
        assert!(symlinked_node_modules.is_symlink());
        assert_eq!(
            fs::read_link(&symlinked_node_modules).unwrap(),
            source.join("node_modules")
        );

        assert_eq!(
            fs::read_to_string(destination.join(".env")).unwrap(),
            "SECRET=1"
        );

        assert!(!destination.join("target").exists());
    }

    #[test]
    fn copy_dir_recursively_copies_nested_directories() {
        let tmp = tempfile::TempDir::new().unwrap();
        let source = tmp.path().join("source");
        let destination = tmp.path().join("destination");

        fs::create_dir_all(source.join("nested")).unwrap();
        fs::write(source.join("nested/file.txt"), "hello").unwrap();
        fs::write(source.join("top.txt"), "top").unwrap();

        copy_dir(&source, &destination).unwrap();

        assert_eq!(
            fs::read_to_string(destination.join("nested/file.txt")).unwrap(),
            "hello"
        );
        assert_eq!(
            fs::read_to_string(destination.join("top.txt")).unwrap(),
            "top"
        );
    }

    #[test]
    fn materialize_path_creates_parent_directories_in_destination() {
        let tmp = tempfile::TempDir::new().unwrap();
        let source = tmp.path().join("source");
        let destination = tmp.path().join("destination");
        fs::create_dir_all(source.join("nested")).unwrap();
        fs::write(source.join("nested/.env"), "SECRET=1").unwrap();

        let config = Config {
            symlinks: PathConfig::default(),
            copies: PathConfig {
                paths: ["**/.env".to_string()].into_iter().collect(),
            },
            ignore: PathConfig::default(),
        };

        let materializer = EnvironmentMaterializer::new(&config);
        let symlink_set = materializer.build_glob_set(&[]).unwrap();
        let copy_set = materializer
            .build_glob_set(&config.copies.paths.iter().cloned().collect::<Vec<_>>())
            .unwrap();
        let ignore_set = materializer.build_glob_set(&[]).unwrap();

        materializer
            .materialize_path(
                &source.join("nested/.env"),
                &source,
                &destination,
                &symlink_set,
                &copy_set,
                &ignore_set,
            )
            .unwrap();

        assert_eq!(
            fs::read_to_string(destination.join("nested/.env")).unwrap(),
            "SECRET=1"
        );
    }
}
