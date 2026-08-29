use std::path::{Path, PathBuf};

use crate::{
    commands::create::Request,
    discovery::project::ProjectIdentity,
    error::DevCloneError,
    materialization::{environment::EnvironmentMaterializer, project::ProjectMaterializer},
};

pub struct Pending;
pub struct ProjectMaterialized;

pub struct Materializer<S> {
    pub request: Request,
    destination: PathBuf,
    _state: S,
}

/// normalize revision for creating the destination directory
///
/// `e.g` docs/new-feature -> docs_new_feature
fn normalize_revision(revision: &str) -> String {
    revision.replace('/', "_").replace("-", "_")
}

fn manage_destination(project: &ProjectIdentity, revision: &str) -> Result<PathBuf, DevCloneError> {
    let parent = project
        .root_path
        .parent()
        .ok_or(DevCloneError::InvalidPath(project.root_path.clone()))?;

    Ok(parent.join(format!("{}_{}", project.name, normalize_revision(revision))))
}

impl<S> Materializer<S> {
    pub fn destination(&self) -> &Path {
        &self.destination
    }
}

impl Materializer<Pending> {
    pub fn new(request: Request) -> Result<Self, DevCloneError> {
        let destination = manage_destination(&request.project, &request.revision)?;

        Ok(Self {
            request,
            destination,
            _state: Pending,
        })
    }

    pub fn materialize_project(self) -> Result<Materializer<ProjectMaterialized>, DevCloneError> {
        let project = ProjectMaterializer::new(&self.request, &self.destination);
        project.materialize()?;

        Ok(Materializer {
            request: self.request,
            destination: self.destination,
            _state: ProjectMaterialized,
        })
    }
}

impl Materializer<ProjectMaterialized> {
    pub fn materialize_environment(self) -> Result<(), DevCloneError> {
        let environment = EnvironmentMaterializer::new(&self.request.config);
        environment.materialize(&self.request.project.root_path, &self.destination)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_revision_replaces_slashes_and_hyphens() {
        assert_eq!(normalize_revision("docs/new-feature"), "docs_new_feature");
        assert_eq!(normalize_revision("main"), "main");
    }

    #[test]
    fn manage_destination_joins_parent_name_and_normalized_revision() {
        let project = ProjectIdentity {
            name: "my-project".to_string(),
            root_path: PathBuf::from("/home/user/my-project"),
        };

        let destination = manage_destination(&project, "feature/x").unwrap();

        assert_eq!(
            destination,
            PathBuf::from("/home/user/my-project_feature_x")
        );
    }

    // This is just because we expect the root path to have a parent, which is true for most cases.
    // If the root path is the root of the filesystem, we can't create a destination directory.
    #[test]
    fn manage_destination_returns_invalid_path_when_root_has_no_parent() {
        let project = ProjectIdentity {
            name: "root".to_string(),
            root_path: PathBuf::from("/"),
        };

        let result = manage_destination(&project, "main");

        assert!(matches!(result, Err(DevCloneError::InvalidPath(_))));
    }
}
