use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use crate::{commands::create::Request, discovery::project::ProjectIdentity, error::DevCloneError};

enum Materialization {
    Archive,
    Git,
}

impl Materialization {
    fn execute(&self, request: &Request, destination: &Path) -> Result<(), DevCloneError> {
        match self {
            Self::Archive => materialize_archive(request, destination),
            Self::Git => materialize_git(request, destination),
        }
    }
}

fn materialize_archive(request: &Request, destination: &Path) -> Result<(), DevCloneError> {
    let mut archive = Command::new("git")
        .arg("-C")
        .arg(&request.project.root_path)
        .arg("archive")
        .arg(&request.revision)
        .stdout(Stdio::piped())
        .spawn()?;

    let archive_stdout = archive.stdout.take().ok_or(DevCloneError::CommandFailed)?;

    let mut tar = Command::new("tar")
        .arg("-x")
        .arg("-C")
        .arg(destination)
        .stdin(archive_stdout)
        .spawn()?;

    let archive_status = archive.wait()?;
    let tar_status = tar.wait()?;

    if !archive_status.success() {
        return Err(DevCloneError::CommandFailed);
    }

    if !tar_status.success() {
        return Err(DevCloneError::CommandFailed);
    }

    Ok(())
}

fn materialize_git(request: &Request, destination: &Path) -> Result<(), DevCloneError> {
    let source = &request.project.root_path;

    run_command(
        Command::new("git")
            .arg("clone")
            .arg("--local")
            .arg(source)
            .arg(destination),
    )?;

    run_command(
        Command::new("git")
            .arg("-C")
            .arg(destination)
            .arg("switch")
            .arg(&request.revision),
    )?;

    Ok(())
}

fn run_command(command: &mut Command) -> Result<(), DevCloneError> {
    let status = command.status()?;

    if !status.success() {
        return Err(DevCloneError::CommandFailed);
    }

    Ok(())
}

fn manage_destination(project: &ProjectIdentity, revision: &str) -> Result<PathBuf, DevCloneError> {
    let parent = project
        .root_path
        .parent()
        .ok_or(DevCloneError::InvalidPath)?;

    Ok(parent.join(format!("{}_{}", project.name, revision)))
}

pub struct Materializer {
    request: Request,
    destination: PathBuf,
}

impl Materializer {
    pub fn new(request: Request) -> Result<Self, DevCloneError> {
        let destination = manage_destination(&request.project, &request.revision)?;

        Ok(Self {
            request,
            destination,
        })
    }

    pub fn materialize(&self) -> Result<(), DevCloneError> {
        let strategy = if self.request.git {
            Materialization::Git
        } else {
            Materialization::Archive
        };

        if !self.request.git {
            fs::create_dir_all(&self.destination)?;
        }

        strategy.execute(&self.request, &self.destination)
    }
}
