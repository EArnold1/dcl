use std::path::PathBuf;

use crate::{
    commands::create::Request,
    discovery::project::ProjectIdentity,
    error::DevCloneError,
    materialization::{environment::EnvironmentMaterializer, project::ProjectMaterializer},
};

pub struct Pending;
pub struct ProjectMaterialized;

pub struct Materializer<S> {
    request: Request,
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
