use crate::{
    config::{loader::Config, paths::Paths},
    error::DevCloneError,
};
use crate::{info, warn};
use std::fs;
use std::io::Write;

pub fn show_config() -> Result<(), DevCloneError> {
    let config = Paths::init()?;
    let config_file = config.config_file();
    let contents = fs::read_to_string(config_file)?;

    println!("{}", contents);
    Ok(())
}

pub fn show_config_path() -> Result<(), DevCloneError> {
    let config = Paths::init()?;
    let config_file = config.config_file();

    println!("{}", config_file.display());
    Ok(())
}

pub fn edit_config() -> Result<(), DevCloneError> {
    let config = Paths::init()?;
    let config_file = config.config_file();

    if !config_file.exists() {
        fs::write(config_file, "")?;
    }

    let editor = resolve_editor()?;
    let status = std::process::Command::new(&editor)
        .arg(config_file)
        .status()
        .map_err(|_| DevCloneError::EditorNotFound(editor))?;

    if !status.success() {
        return Err(DevCloneError::EditorFailed);
    }

    // Validate TOML after edit, warn if invalid so user doesn't lose their work
    if let Err(e) = toml::from_str::<Config>(&fs::read_to_string(config_file)?) {
        warn!("Config file has invalid TOML syntax: {}", e);
        info!("The file has been saved, but run 'dcl config show' to review it.");
    } else {
        info!("Config updated successfully.");
    }

    Ok(())
}

pub fn reset_config(yes: bool) -> Result<(), DevCloneError> {
    if !yes {
        print!(
            "Are you sure you want to reset the configuration? This will overwrite your config. (y/N) "
        );
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            info!("Aborting reset...");
            return Ok(());
        }
    }

    let config = Paths::init()?;
    let config_file = config.config_file();
    let default_config = crate::config::paths::DEFAULT_CONFIG;

    fs::write(config_file, default_config)?;
    info!("Config reset to defaults.");

    Ok(())
}

fn resolve_editor() -> Result<String, DevCloneError> {
    // Check $VISUAL first (for visual editors), then $EDITOR, then platform default
    let editor = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| get_platform_default_editor());
    Ok(editor)
}

fn is_executable_on_path(name: &str) -> bool {
    let Some(path_var) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path_var).any(|dir| {
        let candidate = dir.join(name);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            candidate
                .metadata()
                .map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
                .unwrap_or(false)
        }
        #[cfg(not(unix))]
        {
            candidate.is_file()
        }
    })
}

fn get_platform_default_editor() -> String {
    if is_executable_on_path("nano") {
        "nano".to_string()
    } else if cfg!(windows) {
        "notepad".to_string()
    } else {
        "vi".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_executable_on_path_returns_false_for_nonexistent_binary() {
        assert!(!is_executable_on_path(
            "definitely-not-a-real-binary-xyz123"
        ));
    }

    #[test]
    fn get_platform_default_editor_prefers_nano_if_available() {
        let editor = get_platform_default_editor();
        if is_executable_on_path("nano") {
            assert_eq!(editor, "nano");
        } else if cfg!(windows) {
            assert_eq!(editor, "notepad");
        } else {
            assert_eq!(editor, "vi");
        }
    }

    #[test]
    fn get_platform_default_editor_returns_notepad_on_windows_without_nano() {
        if cfg!(windows) && !is_executable_on_path("nano") {
            let editor = get_platform_default_editor();
            assert_eq!(editor, "notepad");
        }
    }

    #[test]
    fn get_platform_default_editor_returns_vi_on_unix_without_nano() {
        if cfg!(unix) && !is_executable_on_path("nano") {
            let editor = get_platform_default_editor();
            assert_eq!(editor, "vi");
        }
    }
}
