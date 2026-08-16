// use std::println

use crate::{config::loader::Config, discovery::project::ProjectIdentity, error::DevCloneError};

#[derive(Debug)]
struct Request {
    config: Config,
    revision: String,
    project: ProjectIdentity,
    git: bool,
}

pub fn create(revision: String, git: bool) -> Result<(), DevCloneError> {
    let config = Config::load_from_file()?;
    let project = ProjectIdentity::new()?;

    let request = Request {
        config,
        revision,
        project,
        git,
    };

    println!("{:?}", request);

    Ok(())
}
