use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

fn main() {
    // If an error occurs in the main function, print it to stderr and exit.
    if let Err(e) = run() {
        eprintln!("golta-shim error: {}", e);
        exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let current_dir = env::current_dir()?;
    let home_dir = home::home_dir().ok_or("Could not find home directory")?;

    // 1. Determine the Go version to use.
    let version = find_go_version(&current_dir, &home_dir)?;

    // 2. Get arguments for the child process.
    let args: Vec<OsString> = env::args_os().skip(1).collect();

    // 3. Execute the command and get the exit code.
    let exit_code = execute_go(&version, &home_dir, args)?;

    exit(exit_code);
}

/// Finds the active Go version by searching for `.golta.json` and then checking the global default.
fn find_go_version(start_dir: &Path, home_dir: &Path) -> Result<String, Box<dyn Error>> {
    // 1a. Look for a version pinned in the project, traversing up from start_dir.
    let mut current_dir = start_dir.to_path_buf();
    loop {
        let pin_file_path = current_dir.join(".golta.json");
        if pin_file_path.exists() {
            let content = fs::read_to_string(pin_file_path)?;
            let json: serde_json::Value = serde_json::from_str(&content)?;
            if let Some(go_ver) = json.get("go").and_then(|v| v.as_str()) {
                return Ok(go_ver.to_string());
            }
        }

        // 1b. Look for a version pinned in go.mod
        let go_mod_path = current_dir.join("go.mod");
        if go_mod_path.exists() {
            let content = fs::read_to_string(go_mod_path)?;
            let mut go_version = None;
            let mut toolchain_version = None;

            for line in content.lines() {
                // Strip comments and whitespace.
                let line = line.split("//").next().unwrap_or("").trim();
                if line.is_empty() {
                    continue;
                }

                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() < 2 {
                    continue;
                }

                match parts[0] {
                    "go" => go_version = Some(parts[1].to_string()),
                    "toolchain" => {
                        // toolchain usually looks like "go1.21.5", so we strip "go".
                        let v = parts[1].strip_prefix("go").unwrap_or(parts[1]);
                        toolchain_version = Some(v.to_string());
                    }
                    _ => {}
                }
            }

            // The toolchain directive takes precedence over the go directive.
            if let Some(v) = toolchain_version.or(go_version) {
                return Ok(v);
            }
        }

        if !current_dir.pop() {
            break; // Reached the root directory.
        }
    }

    // 1b. If not pinned, read the global default version.
    let default_file = home_dir.join(".golta").join("state").join("default.txt");
    let version = fs::read_to_string(default_file).unwrap_or_default();
    let version = version.trim().to_string();

    if version.is_empty() {
        Err("No Go version is set. Use `golta pin go@<version>` in your project, or `golta default go@<version>` globally.".into())
    } else {
        Ok(version)
    }
}

/// Constructs the path to the Go executable and runs it.
fn execute_go(version: &str, home_dir: &Path, args: Vec<OsString>) -> Result<i32, Box<dyn Error>> {
    let go_executable_name = if cfg!(windows) { "go.exe" } else { "go" };
    let version_number = version.trim_start_matches("go@");
    let real_go_path: PathBuf = home_dir
        .join(".golta")
        .join("versions")
        .join(version_number)
        .join("bin")
        .join(go_executable_name);

    if !real_go_path.exists() {
        // Check for CI environment or explicit auto-install flag to avoid blocking.
        let is_ci = env::var("CI").is_ok();
        let auto_install = env::var("GOLTA_AUTO_INSTALL")
            .map(|v| v == "1" || v == "true")
            .unwrap_or(false);

        if is_ci && !auto_install {
            return Err(format!(
                "Go version {} is not installed. CI detected, aborting.",
                version_number
            )
            .into());
        }

        let should_install = if auto_install {
            true
        } else {
            eprintln!("Go version {} is not installed.", version_number);
            eprint!("Would you like to install it? [Y/n] ");
            io::stderr().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim().to_lowercase();
            input == "" || input == "y" || input == "yes"
        };

        if should_install {
            let status = Command::new("golta").arg("install").arg(version).status()?;

            if !status.success() {
                return Err(format!("Failed to install Go version {}.", version).into());
            }
        } else {
            return Err(format!("Go version {} is not installed.", version).into());
        }
    }

    let mut command = Command::new(&real_go_path);
    command.args(args);

    // Set GOROOT to ensure the correct toolchain is used, avoiding conflicts with system GOROOT.
    if let Some(bin_dir) = real_go_path.parent() {
        if let Some(go_root) = bin_dir.parent() {
            command.env("GOROOT", go_root);
        }
    }

    #[cfg(unix)]
    {
        let err = command.exec();
        return Err(err.into());
    }

    #[cfg(not(unix))]
    {
        let status = command.status()?;
        Ok(status.code().unwrap_or(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_find_go_version_no_config() {
        let project_dir = tempdir().unwrap();
        let home_dir = tempdir().unwrap();

        let result = find_go_version(project_dir.path(), home_dir.path());

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "No Go version is set. Use `golta pin go@<version>` in your project, or `golta default go@<version>` globally."
        );
    }

    #[test]
    fn test_find_go_version_with_pin_file() {
        let project_dir = tempdir().unwrap();
        let home_dir = tempdir().unwrap();
        let pin_file = project_dir.path().join(".golta.json");
        fs::write(pin_file, r#"{"go": "1.21.0"}"#).unwrap();

        let result = find_go_version(project_dir.path(), home_dir.path());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "1.21.0");
    }

    #[test]
    fn test_find_go_version_with_pin_file_in_parent_dir() {
        let project_dir = tempdir().unwrap();
        let sub_dir = project_dir.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();
        let home_dir = tempdir().unwrap();
        let pin_file = project_dir.path().join(".golta.json");
        fs::write(pin_file, r#"{"go": "1.22.0"}"#).unwrap();

        let result = find_go_version(&sub_dir, home_dir.path());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "1.22.0");
    }

    #[test]
    fn test_find_go_version_with_default_file() {
        let project_dir = tempdir().unwrap();
        let home_dir = tempdir().unwrap();
        let state_dir = home_dir.path().join(".golta").join("state");
        fs::create_dir_all(&state_dir).unwrap();
        fs::write(state_dir.join("default.txt"), "1.20.5").unwrap();

        let result = find_go_version(project_dir.path(), home_dir.path());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "1.20.5");
    }

    #[test]
    fn test_find_go_version_pin_overrides_default() {
        let project_dir = tempdir().unwrap();
        let pin_file = project_dir.path().join(".golta.json");
        fs::write(pin_file, r#"{"go": "1.21.0"}"#).unwrap();

        let home_dir = tempdir().unwrap();
        let state_dir = home_dir.path().join(".golta").join("state");
        fs::create_dir_all(&state_dir).unwrap();
        fs::write(state_dir.join("default.txt"), "1.20.5").unwrap();

        let result = find_go_version(project_dir.path(), home_dir.path());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "1.21.0");
    }

    #[test]
    fn test_find_go_version_with_go_mod() {
        let project_dir = tempdir().unwrap();
        let home_dir = tempdir().unwrap();
        let go_mod = project_dir.path().join("go.mod");
        fs::write(go_mod, "module example.com/test\n\ngo 1.23.0\n").unwrap();

        let result = find_go_version(project_dir.path(), home_dir.path());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "1.23.0");
    }

    #[test]
    fn test_find_go_version_with_go_mod_comments() {
        let project_dir = tempdir().unwrap();
        let home_dir = tempdir().unwrap();
        let go_mod = project_dir.path().join("go.mod");
        fs::write(
            go_mod,
            "module example.com/test\n\n// comment\ngo 1.23.0 // inline comment\n",
        )
        .unwrap();

        let result = find_go_version(project_dir.path(), home_dir.path());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "1.23.0");
    }

    #[test]
    fn test_find_go_version_with_toolchain() {
        let project_dir = tempdir().unwrap();
        let home_dir = tempdir().unwrap();
        let go_mod = project_dir.path().join("go.mod");
        fs::write(
            go_mod,
            "module example.com/test\n\ngo 1.21\ntoolchain go1.21.5\n",
        )
        .unwrap();

        let result = find_go_version(project_dir.path(), home_dir.path());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "1.21.5");
    }
}
