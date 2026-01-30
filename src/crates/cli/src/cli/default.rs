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
            // Currently, the CLI structure for `Clear` implies "go", but the manager is generic.
            if manager.clear_default("go")? {
                println!("Cleared global default Go version.");
            } else {
                println!("No global default Go version is set.");
            }
        }
        None => {
            let tool_arg = cmd.tool.expect("clap should ensure tool is present");
            let (tool, version) = parse_tool_version(&tool_arg)?;
            manager.set_default(tool, version)?;
            println!("Set {} default version to {}", tool, version);
        }
    }
    Ok(())
}

fn parse_tool_version(input: &str) -> Result<(&str, &str), Box<dyn Error>> {
    let (tool, version) = input
        .split_once('@')
        .ok_or("Invalid format. Use <tool>@<version>.")?;
    Ok((tool, version))
}

/// `DefaultManager` abstracts the operations for managing the default version.
/// This decouples the dependency on the filesystem and makes testing easier.
trait DefaultManager {
    /// Sets the specified version as the default.
    /// Returns an error if the version is not installed.
    fn set_default(&mut self, tool: &str, version: &str) -> Result<(), Box<dyn Error>>;

    /// Clears the currently set default version.
    /// Returns `Ok(false)` if no default was set.
    fn clear_default(&mut self, tool: &str) -> Result<bool, Box<dyn Error>>;
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

    fn default_file_path(&self, tool: &str) -> PathBuf {
        if tool == "go" {
            self.state_dir.join("default.txt")
        } else {
            self.state_dir.join(format!("{}.default", tool))
        }
    }
}

impl DefaultManager for FsDefaultManager {
    fn set_default(&mut self, tool: &str, version: &str) -> Result<(), Box<dyn Error>> {
        // Check if the version is installed (currently only validates 'go')
        if tool == "go" {
            let installed_versions = get_installed_versions()?;
            if !installed_versions.iter().any(|v| v == version) {
                return Err(format!(
                    "Go version {} is not installed. Please install it first with `golta install go@{}`.",
                    version, version
                )
                .into());
            }
        }
        // TODO: Add validation for other tools when directory structure supports them

        create_dir_all(&self.state_dir)?;
        write(self.default_file_path(tool), version)?;

        Ok(())
    }

    fn clear_default(&mut self, tool: &str) -> Result<bool, Box<dyn Error>> {
        let default_file = self.default_file_path(tool);
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
        fn set_default(&mut self, tool: &str, version: &str) -> Result<(), Box<dyn Error>> {
            if tool == "go" && !self.installed_versions.contains(version) {
                return Err("Version not installed".into());
            }
            self.default_version = Some(version.to_string());
            Ok(())
        }

        fn clear_default(&mut self, _tool: &str) -> Result<bool, Box<dyn Error>> {
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
        assert_eq!(parse_tool_version("go@1.21.0").unwrap(), ("go", "1.21.0"));
    }

    #[test]
    fn test_parse_tool_version_other_tool() {
        assert_eq!(parse_tool_version("air@1.49.0").unwrap(), ("air", "1.49.0"));
    }
}
