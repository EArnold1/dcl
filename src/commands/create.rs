use crate::{
    config::loader::Config, discovery::project::ProjectIdentity, error::DevCloneError,
    materialization::materializer,
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

    let request = Request {
        config,
        revision,
        project,
        git,
    };

    materializer::Materializer::new(request)?
        .materialize_project()?
        .materialize_environment()?;

    Ok(())
}
