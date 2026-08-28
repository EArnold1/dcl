use std::fs;

use dcl::{
    config::loader::{Config, PathConfig},
    materialization::environment::EnvironmentMaterializer,
};
use tempfile::TempDir;

#[test]
fn materialize_symlinks_copies_and_ignores_configured_paths() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("source");
    let destination = tmp.path().join("destination");
    fs::create_dir_all(&source).unwrap();

    fs::create_dir_all(source.join("node_modules/react")).unwrap();
    fs::write(source.join("node_modules/react/index.js"), "react").unwrap();

    fs::write(source.join(".env"), "SECRET=1").unwrap();

    fs::create_dir_all(source.join("target")).unwrap();
    fs::write(source.join("target/output"), "binary").unwrap();

    fs::write(source.join("README.md"), "# project").unwrap();

    let config = Config {
        symlinks: PathConfig {
            paths: ["**/node_modules".to_string()].into_iter().collect(),
        },
        copies: PathConfig {
            paths: ["**/.env".to_string()].into_iter().collect(),
        },
        ignore: PathConfig {
            paths: ["**/target".to_string()].into_iter().collect(),
        },
    };

    EnvironmentMaterializer::new(&config)
        .materialize(&source, &destination)
        .unwrap();

    let symlinked_node_modules = destination.join("node_modules");
    assert!(symlinked_node_modules.is_symlink());
    assert_eq!(
        fs::read_link(&symlinked_node_modules).unwrap(),
        source.join("node_modules")
    );

    assert_eq!(
        fs::read_to_string(destination.join(".env")).unwrap(),
        "SECRET=1"
    );

    // ignored path is never materialized, even though it exists in source
    assert!(!destination.join("target").exists());

    // paths not covered by any symlink/copy/ignore pattern are left untouched
    assert!(!destination.join("README.md").exists());
}

#[test]
fn materialize_copies_nested_directories_recursively() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("source");
    let destination = tmp.path().join("destination");
    fs::create_dir_all(source.join(".cargo")).unwrap();
    fs::write(
        source.join(".cargo/config.toml"),
        "[build]\ntarget = \"x86_64\"",
    )
    .unwrap();

    let config = Config {
        symlinks: PathConfig::default(),
        copies: PathConfig {
            paths: ["**/.cargo/config.toml".to_string()].into_iter().collect(),
        },
        ignore: PathConfig::default(),
    };

    EnvironmentMaterializer::new(&config)
        .materialize(&source, &destination)
        .unwrap();

    assert_eq!(
        fs::read_to_string(destination.join(".cargo/config.toml")).unwrap(),
        "[build]\ntarget = \"x86_64\""
    );
}
