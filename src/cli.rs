use clap::{Parser, Subcommand};

use crate::{commands::create::create, error::DevCloneError};

#[derive(Debug, Parser)]
#[command(name = "dcl")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(alias = "c")]
    Create {
        revision: String,

        #[arg(long)]
        git: bool,
    },
}

pub fn run() -> Result<(), DevCloneError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create { revision, git } => create(revision, git)?,
    }

    Ok(())
}
