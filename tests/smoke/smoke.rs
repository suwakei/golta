use std::env;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

// Go versions to use for testing
const GO_VERSION: &str = "1.22.0";
const GO_VERSION_ALT: &str = "1.21.0";

/// A struct to manage setup and cleanup for smoke tests
struct SmokeTest {
    /// A temporary home directory for `golta` to use
    home_dir: TempDir,
    /// Records the versions installed during the test
    installed_versions: Vec<String>,
    /// The path to the `golta` executable
    golta_bin: String,
}

impl SmokeTest {
    /// Sets up the test environment
    fn setup() -> Self {
        // Get the path to the `golta` binary built by `cargo test`
        let golta_bin = env!("CARGO_BIN_EXE_golta").to_string();
        // Create a temporary home directory for testing
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let mut test = SmokeTest {
            home_dir: temp_dir,
            installed_versions: Vec::new(),
            golta_bin,
        };

        // Install the Go versions to be used in the test
        test.install_version(GO_VERSION);
        test.install_version(GO_VERSION_ALT);

        test
    }

    /// Helper to execute a `golta` command
    fn golta(&self, args: &[&str]) -> Output {
        Command::new(&self.golta_bin)
            .args(args)
            .env("HOME", self.home_dir.path())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Failed to execute command")
    }

    /// Helper to execute a `golta` command in a specific directory
    fn golta_in_dir(&self, args: &[&str], dir: &Path) -> Output {
        Command::new(&self.golta_bin)
            .args(args)
            .current_dir(dir)
            .env("HOME", self.home_dir.path())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Failed to execute command in dir")
    }

    /// Helper to install a specific version
    fn install_version(&mut self, version: &str) {
        let tool_version = format!("go@{}", version);
        let output = self.golta(&["install", &tool_version]);

        assert!(
            output.status.success(),
            "install command for {} failed: {}",
            version,
            String::from_utf8_lossy(&output.stderr)
        );
        self.installed_versions.push(tool_version);
    }
}

impl Drop for SmokeTest {
    /// Runs `uninstall` to clean up when the test finishes
    fn drop(&mut self) {
        // Clear `default` before uninstalling
        self.golta(&["default", "clear"]);

        for tool_version in &self.installed_versions {
            let output = self.golta(&["uninstall", tool_version]);
            // Continue with other cleanups even if one fails
            if !output.status.success() {
                eprintln!(
                    "Failed to uninstall {}: {}",
                    tool_version,
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
    }
}

#[test]
fn test_exec_go_version() {
    let test = SmokeTest::setup();

    // Set the `default` version
    let tool_version = format!("go@{}", GO_VERSION);
    test.golta(&["default", &tool_version]);

    // Check the version with `exec`
    let output = test.golta(&["exec", "go", "version"]);
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains(&format!("go version go{}", GO_VERSION)));
}

#[test]
fn test_pin_and_unpin() {
    let test = SmokeTest::setup();

    // Set the global default
    let default_tool_version = format!("go@{}", GO_VERSION);
    test.golta(&["default", &default_tool_version]);

    // Create a subdirectory for the project
    let project_dir = test.home_dir.path().join("my-project");
    std::fs::create_dir(&project_dir).unwrap();

    // Pin a different version to the project
    let pin_tool_version = format!("go@{}", GO_VERSION_ALT);
    let output = test.golta_in_dir(&["pin", &pin_tool_version], &project_dir);
    assert!(output.status.success());
    assert!(project_dir.join(".golta.json").exists());

    // Confirm that the pinned version is used within the project
    let output = test.golta_in_dir(&["exec", "go", "version"], &project_dir);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(&format!("go version go{}", GO_VERSION_ALT)));

    // Unpin the version
    let output = test.golta_in_dir(&["unpin"], &project_dir);
    assert!(output.status.success());
    assert!(!project_dir.join(".golta.json").exists());

    // Confirm that it reverts to the global default within the project
    let output = test.golta_in_dir(&["exec", "go", "version"], &project_dir);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(&format!("go version go{}", GO_VERSION)));
}

#[test]
fn test_which_shows_correct_path() {
    let test = SmokeTest::setup();

    // Set the `default` version
    let tool_version = format!("go@{}", GO_VERSION);
    test.golta(&["default", &tool_version]);

    // Check the path with `which`
    let output = test.golta(&["which", "go"]);
    let stdout = String::from_utf8(output.stdout).unwrap();

    let expected_path_fragment = test
        .home_dir
        .path()
        .join(".golta")
        .join("versions")
        .join(GO_VERSION);

    assert!(stdout.contains(expected_path_fragment.to_str().unwrap()));
}

#[test]
fn test_cannot_uninstall_default_version() {
    let test = SmokeTest::setup();

    let tool_version = format!("go@{}", GO_VERSION);
    test.golta(&["default", &tool_version]);

    let output = test.golta(&["uninstall", &tool_version]);
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("because it is the default version"));
}
