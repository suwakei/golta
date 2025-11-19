use std::env;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

// Go versions to use for testing
pub const GO_VERSION: &str = "1.22.0";
pub const GO_VERSION_ALT: &str = "1.21.0";

/// A struct to manage setup and cleanup for smoke tests
pub struct SmokeTest {
    /// A temporary home directory for `golta` to use
    home_dir: TempDir,
    /// Records the versions installed during the test
    installed_versions: Vec<String>,
    /// The path to the `golta` executable
    golta_bin: String,
}

impl SmokeTest {
    /// Sets up the test environment
    pub fn setup() -> Self {
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

    /// Returns a reference to the temporary home directory.
    pub fn home_dir(&self) -> &TempDir {
        &self.home_dir
    }

    /// Helper to execute a `golta` command
    pub fn golta(&self, args: &[&str]) -> Output {
        Command::new(&self.golta_bin)
            .args(args)
            .env("HOME", self.home_dir.path())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Failed to execute command")
    }

    /// Helper to execute a `golta` command in a specific directory
    pub fn golta_in_dir(&self, args: &[&str], dir: &Path) -> Output {
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
