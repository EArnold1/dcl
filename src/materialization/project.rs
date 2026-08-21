use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
};

use crate::{commands::create::Request, error::DevCloneError, info};

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

    let archive_stdout = archive.stdout.take().ok_or(DevCloneError::CommandFailed {
        command: "git archive".into(),
        stderr: "Failed to capture stdout".into(),
    })?;

    let mut tar = Command::new("tar")
        .arg("-x")
        .arg("-C")
        .arg(destination)
        .stdin(archive_stdout)
        .spawn()?;

    let archive_status = archive.wait()?;
    let tar_status = tar.wait()?;

    if !archive_status.success() {
        return Err(DevCloneError::CommandFailed {
            command: "git archive".into(),
            stderr: "git archive failed".into(),
        });
    }

    if !tar_status.success() {
        return Err(DevCloneError::CommandFailed {
            command: "tar extraction".into(),
            stderr: "tar extraction failed".into(),
        });
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
        "git clone",
    )?;

    run_command(
        Command::new("git")
            .arg("-C")
            .arg(destination)
            .arg("switch")
            .arg(&request.revision),
        "git switch",
    )?;

    Ok(())
}

fn run_command(command: &mut Command, action: &str) -> Result<(), DevCloneError> {
    if let Err(err) = command.status() {
        return Err(DevCloneError::CommandFailed {
            command: action.into(),
            stderr: err.to_string(),
        });
    }

    Ok(())
}

pub struct ProjectMaterializer<'a> {
    request: &'a Request,
    destination: &'a Path,
}

impl<'a> ProjectMaterializer<'a> {
    pub fn new(request: &'a Request, destination: &'a Path) -> Self {
        Self {
            request,
            destination,
        }
    }

    pub fn materialize(&self) -> Result<(), DevCloneError> {
        if self.destination.exists() {
            return Err(DevCloneError::DestinationExists(self.destination.into()));
        }

        if !self.request.git {
            match fs::create_dir_all(self.destination) {
                Ok(_) => {
                    info!("Created destination directory: {:?}", &self.destination);
                }
                Err(e) => return Err(DevCloneError::Io(e)),
            }
        }

        let strategy = if self.request.git {
            Materialization::Git
        } else {
            Materialization::Archive
        };

        strategy.execute(self.request, self.destination)
    }
}
