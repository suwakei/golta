use crate::shared::local_versions::get_installed_versions;
use crate::DefaultCommand;
use std::error::Error;
use std::fs::{create_dir_all, remove_file, write};
use std::path::PathBuf;

pub fn run(cmd: DefaultCommand) {
    if let Err(e) = handle_default(cmd, &mut FsDefaultManager::new()) {
        eprintln!("Error: {}", e);
    }
}

fn handle_default(
    cmd: DefaultCommand,
    manager: &mut impl DefaultManager,
) -> Result<(), Box<dyn Error>> {
    match cmd.command {
        Some(crate::DefaultCommands::Clear) => {
            if manager.clear_default()? {
                println!("Cleared global default Go version.");
            } else {
                println!("No global default Go version is set.");
            }
        }
        None => {
            let tool = cmd.tool.expect("clap should ensure tool is present");
            let version = parse_tool_version(&tool)?;
            manager.set_default(version)?;
            println!("Set Go default version to {}", version);
        }
    }
    Ok(())
}

fn parse_tool_version(tool: &str) -> Result<&str, Box<dyn Error>> {
    if !tool.starts_with("go@") {
        return Err(
            "Only Go default version is supported currently. Use format 'go@<version>'.".into(),
        );
    }
    Ok(tool.trim_start_matches("go@"))
}

/// `DefaultManager` abstracts the operations for managing the default version.
/// This decouples the dependency on the filesystem and makes testing easier.
trait DefaultManager {
    /// Sets the specified version as the default.
    /// Returns an error if the version is not installed.
    fn set_default(&mut self, version: &str) -> Result<(), Box<dyn Error>>;

    /// Clears the currently set default version.
    /// Returns `Ok(false)` if no default was set.
    fn clear_default(&mut self) -> Result<bool, Box<dyn Error>>;
}

/// Filesystem implementation of `DefaultManager`.
struct FsDefaultManager {
    state_dir: PathBuf,
}

impl FsDefaultManager {
    fn new() -> Self {
        let home = home::home_dir().expect("Could not find home directory");
        let state_dir = home.join(".golta").join("state");
        Self { state_dir }
    }

    fn default_file_path(&self) -> PathBuf {
        self.state_dir.join("default.txt")
    }
}

impl DefaultManager for FsDefaultManager {
    fn set_default(&mut self, version: &str) -> Result<(), Box<dyn Error>> {
        // Check if the version is installed
        let installed_versions = get_installed_versions()?;
        if !installed_versions.iter().any(|v| v == version) {
            return Err(format!(
                "Go version {} is not installed. Please install it first with `golta install go@{}`.",
                version, version
            )
            .into());
        }

        create_dir_all(&self.state_dir)?;
        write(self.default_file_path(), version)?;

        Ok(())
    }

    fn clear_default(&mut self) -> Result<bool, Box<dyn Error>> {
        let default_file = self.default_file_path();
        if default_file.exists() {
            remove_file(default_file)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DefaultCommand, DefaultCommands};
    use std::collections::HashSet;

    struct MockManager {
        installed_versions: HashSet<String>,
        default_version: Option<String>,
    }

    impl MockManager {
        fn new(installed_versions: Vec<&str>, default_version: Option<&str>) -> Self {
            Self {
                installed_versions: installed_versions.into_iter().map(String::from).collect(),
                default_version: default_version.map(String::from),
            }
        }
    }

    impl DefaultManager for MockManager {
        fn set_default(&mut self, version: &str) -> Result<(), Box<dyn Error>> {
            if !self.installed_versions.contains(version) {
                return Err("Version not installed".into());
            }
            self.default_version = Some(version.to_string());
            Ok(())
        }

        fn clear_default(&mut self) -> Result<bool, Box<dyn Error>> {
            if self.default_version.is_some() {
                self.default_version = None;
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }

    #[test]
    fn test_handle_default_set_version_success() {
        let mut manager = MockManager::new(vec!["1.21.0"], None);
        let cmd = DefaultCommand {
            command: None,
            tool: Some("go@1.21.0".to_string()),
        };

        let result = handle_default(cmd, &mut manager); // この行は変更不要ですが、関数の定義が変わったことで正しく動作します
        assert!(result.is_ok());
        assert_eq!(manager.default_version, Some("1.21.0".to_string()));
    }

    #[test]
    fn test_handle_default_set_version_not_installed() {
        let mut manager = MockManager::new(vec!["1.20.0"], None);
        let cmd = DefaultCommand {
            command: None,
            tool: Some("go@1.21.0".to_string()),
        };

        let result = handle_default(cmd, &mut manager); // この行は変更不要
        assert!(result.is_err());
        assert_eq!(manager.default_version, None);
    }

    #[test]
    fn test_handle_default_clear() {
        let mut manager = MockManager::new(vec!["1.21.0"], Some("1.21.0"));
        let cmd = DefaultCommand {
            command: Some(DefaultCommands::Clear),
            tool: None,
        };

        let result = handle_default(cmd, &mut manager); // この行は変更不要
        assert!(result.is_ok());
        assert_eq!(manager.default_version, None);
    }

    #[test]
    fn test_parse_tool_version_success() {
        assert_eq!(parse_tool_version("go@1.21.0").unwrap(), "1.21.0");
    }

    #[test]
    fn test_parse_tool_version_invalid() {
        assert!(parse_tool_version("node@18").is_err());
    }
}
