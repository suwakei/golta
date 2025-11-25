use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
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
    let args: Vec<String> = env::args_os()
        .skip(1)
        .map(|s| s.into_string().unwrap())
        .collect();

    // 3. Execute the command and get the exit code.
    let exit_code = execute_go(&version, &home_dir, args)?;

    exit(exit_code);
}

/// Finds the active Go version by searching for `.golta.json` and then checking the global default.
fn find_go_version(start_dir: &PathBuf, home_dir: &PathBuf) -> Result<String, Box<dyn Error>> {
    // 1a. Look for a version pinned in the project, traversing up from start_dir.
    let mut current_dir = start_dir.clone();
    loop {
        let pin_file_path = current_dir.join(".golta.json");
        if pin_file_path.exists() {
            let content = fs::read_to_string(pin_file_path)?;
            let json: serde_json::Value = serde_json::from_str(&content)?;
            if let Some(go_ver) = json.get("go").and_then(|v| v.as_str()) {
                return Ok(go_ver.to_string());
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
fn execute_go(version: &str, home_dir: &PathBuf, args: Vec<String>) -> Result<i32, Box<dyn Error>> {
    let go_executable_name = if cfg!(windows) { "go.exe" } else { "go" };
    let real_go_path: PathBuf = home_dir
        .join(".golta")
        .join("versions")
        .join(version.trim_start_matches("go@")) // Remove the "go@" prefix.
        .join("bin")
        .join(go_executable_name);

    // Execute the actual Go binary.
    let status = Command::new(real_go_path).args(args).status()?;

    // Return the exit code of the child process.
    Ok(status.code().unwrap_or(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_find_go_version_no_config() {
        let project_dir = tempdir().unwrap();
        let home_dir = tempdir().unwrap();

        let result = find_go_version(
            &project_dir.path().to_path_buf(),
            &home_dir.path().to_path_buf(),
        );

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

        let result = find_go_version(
            &project_dir.path().to_path_buf(),
            &home_dir.path().to_path_buf(),
        );

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

        let result = find_go_version(&sub_dir, &home_dir.path().to_path_buf());

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

        let result = find_go_version(
            &project_dir.path().to_path_buf(),
            &home_dir.path().to_path_buf(),
        );

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

        let result = find_go_version(
            &project_dir.path().to_path_buf(),
            &home_dir.path().to_path_buf(),
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "1.21.0");
    }
}
