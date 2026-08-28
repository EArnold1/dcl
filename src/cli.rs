use clap::{Parser, Subcommand};

use crate::{
    commands::{create::create, list::list_instances},
    error::DevCloneError,
};

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
    #[command(alias = "ls")]
    List,
}

pub fn run() -> Result<(), DevCloneError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create { revision, git } => create(revision, git)?,
        Commands::List => list_instances()?,
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_parses_revision_without_git_flag() {
        let cli = Cli::try_parse_from(["dcl", "create", "main"]).unwrap();

        assert!(matches!(
            cli.command,
            Commands::Create { revision, git } if revision == "main" && !git
        ));
    }

    #[test]
    fn create_parses_git_flag() {
        let cli = Cli::try_parse_from(["dcl", "create", "main", "--git"]).unwrap();

        assert!(matches!(
            cli.command,
            Commands::Create { revision, git } if revision == "main" && git
        ));
    }

    #[test]
    fn create_alias_c_parses_same_as_create() {
        let cli = Cli::try_parse_from(["dcl", "c", "docs/new-feature"]).unwrap();

        assert!(matches!(
            cli.command,
            Commands::Create { revision, git } if revision == "docs/new-feature" && !git
        ));
    }

    #[test]
    fn create_without_revision_fails_to_parse() {
        let result = Cli::try_parse_from(["dcl", "create"]);

        assert!(result.is_err());
    }

    #[test]
    fn unknown_subcommand_fails_to_parse() {
        let result = Cli::try_parse_from(["dcl", "bogus"]);

        assert!(result.is_err());
    }
}
