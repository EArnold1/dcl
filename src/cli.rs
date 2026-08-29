use clap::{Parser, Subcommand};

use crate::{
    commands::{config, create::create, list::list_instances, remove::remove},
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
    #[command(alias = "rm")]
    Remove {
        target: String,

        #[arg(short = 'y', long)]
        yes: bool,
    },
    #[command(alias = "cfg")]
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Debug, Subcommand)]
enum ConfigAction {
    Show,
    Path,
    Edit,
    Reset {
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

pub fn run() -> Result<(), DevCloneError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create { revision, git } => create(revision, git)?,
        Commands::List => list_instances()?,
        Commands::Remove { target, yes } => remove(target, yes)?,
        Commands::Config { action } => match action {
            ConfigAction::Show => config::show_config()?,
            ConfigAction::Path => config::show_config_path()?,
            ConfigAction::Edit => config::edit_config()?,
            ConfigAction::Reset { yes } => config::reset_config(yes)?,
        },
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

    #[test]
    fn list_parses_without_arguments() {
        let cli = Cli::try_parse_from(["dcl", "list"]).unwrap();

        assert!(matches!(cli.command, Commands::List));
    }

    #[test]
    fn list_alias_ls_parses_same_as_list() {
        let cli = Cli::try_parse_from(["dcl", "ls"]).unwrap();

        assert!(matches!(cli.command, Commands::List));
    }

    #[test]
    fn remove_parses_target_without_yes_flag() {
        let cli = Cli::try_parse_from(["dcl", "remove", "my_instance"]).unwrap();

        assert!(matches!(
            cli.command,
            Commands::Remove { target, yes } if target == "my_instance" && !yes
        ));
    }

    #[test]
    fn remove_parses_yes_flag() {
        let cli = Cli::try_parse_from(["dcl", "remove", "my_instance", "--yes"]).unwrap();

        assert!(matches!(
            cli.command,
            Commands::Remove { target, yes } if target == "my_instance" && yes
        ));
    }

    #[test]
    fn remove_parses_y_flag() {
        let cli = Cli::try_parse_from(["dcl", "remove", "my_instance", "-y"]).unwrap();

        assert!(matches!(
            cli.command,
            Commands::Remove { target, yes } if target == "my_instance" && yes
        ));
    }

    #[test]
    fn remove_alias_rm_parses_same_as_remove() {
        let cli = Cli::try_parse_from(["dcl", "rm", "my_instance", "-y"]).unwrap();

        assert!(matches!(
            cli.command,
            Commands::Remove { target, yes } if target == "my_instance" && yes
        ));
    }

    #[test]
    fn remove_without_target_fails_to_parse() {
        let result = Cli::try_parse_from(["dcl", "remove"]);

        assert!(result.is_err());
    }

    #[test]
    fn config_show_parses() {
        let cli = Cli::try_parse_from(["dcl", "config", "show"]).unwrap();

        assert!(matches!(
            cli.command,
            Commands::Config {
                action: ConfigAction::Show
            }
        ));
    }

    #[test]
    fn config_path_parses() {
        let cli = Cli::try_parse_from(["dcl", "config", "path"]).unwrap();

        assert!(matches!(
            cli.command,
            Commands::Config {
                action: ConfigAction::Path
            }
        ));
    }

    #[test]
    fn config_edit_parses() {
        let cli = Cli::try_parse_from(["dcl", "config", "edit"]).unwrap();

        assert!(matches!(
            cli.command,
            Commands::Config {
                action: ConfigAction::Edit
            }
        ));
    }

    #[test]
    fn config_reset_parses_with_yes_flag() {
        let cli = Cli::try_parse_from(["dcl", "config", "reset", "-y"]).unwrap();

        assert!(matches!(
            cli.command,
            Commands::Config {
                action: ConfigAction::Reset { yes: true }
            }
        ));
    }

    #[test]
    fn config_reset_parses_with_long_yes_flag() {
        let cli = Cli::try_parse_from(["dcl", "config", "reset", "--yes"]).unwrap();

        assert!(matches!(
            cli.command,
            Commands::Config {
                action: ConfigAction::Reset { yes: true }
            }
        ));
    }

    #[test]
    fn config_reset_without_yes_flag_defaults_to_false() {
        let cli = Cli::try_parse_from(["dcl", "config", "reset"]).unwrap();

        assert!(matches!(
            cli.command,
            Commands::Config {
                action: ConfigAction::Reset { yes: false }
            }
        ));
    }

    #[test]
    fn config_alias_cfg_parses_same_as_config() {
        let cli = Cli::try_parse_from(["dcl", "cfg", "show"]).unwrap();

        assert!(matches!(
            cli.command,
            Commands::Config {
                action: ConfigAction::Show
            }
        ));
    }
}
