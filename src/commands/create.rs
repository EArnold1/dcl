use crate::{
    config::loader::Config,
    discovery::project::ProjectIdentity,
    error::DevCloneError,
    materialization::{Materialization, materializer},
    registry::{Instance, InstanceStatus, Registry},
};

#[derive(Debug)]
pub struct Request {
    pub config: Config,
    pub revision: String,
    pub project: ProjectIdentity,
    pub git: bool,
}

pub fn create(revision: String, git: bool) -> Result<(), DevCloneError> {
    let config = Config::load_from_file()?;
    let project = ProjectIdentity::discover()?;

    let source = project.root_path.clone();
    let recorded_revision = revision.clone();
    let mode = Materialization::from_flag(git);

    let request = Request {
        config,
        revision,
        project,
        git,
    };

    let materializer = materializer::Materializer::new(request)?;
    let destination = materializer.destination().to_path_buf();

    let name = destination
        .file_name()
        .ok_or(DevCloneError::InvalidPath(destination.clone()))?
        .to_string_lossy()
        .into_owned();

    let instance = Instance::new(name, destination.clone(), source, recorded_revision, mode);

    let mut registry = Registry::load()?;
    registry.add(instance);
    registry.save()?;

    let result = materializer
        .materialize_project()
        .and_then(|m| m.materialize_environment());

    if let Some(entry) = registry.find_mut_by_destination(&destination) {
        entry.status = match &result {
            Ok(()) => InstanceStatus::Ready,
            Err(err) => InstanceStatus::Failed {
                reason: err.to_string(),
            }, // TODO: during failures, already generated instances should be cleaned up
        };
    }
    registry.save()?;

    result
}
