mod cli;
pub mod commands;
pub mod config;
pub mod discovery;
pub mod error;
pub mod logger;
pub mod materialization;

fn main() {
    if let Err(err) = cli::run() {
        lerror!("{err}");
        std::process::exit(1);
    }
}
