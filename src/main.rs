use dcl::cli;

fn main() {
    if let Err(err) = cli::run() {
        dcl::lerror!("{err}");
        std::process::exit(1);
    }
}
