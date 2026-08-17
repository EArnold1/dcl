mod cli;
pub mod commands;
pub mod config;
pub mod discovery;
pub mod error;
pub mod materialization;

fn main() -> Result<(), error::DevCloneError> {
    cli::run()?;
    Ok(())
}
