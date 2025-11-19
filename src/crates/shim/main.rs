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
    // 1. Determine the Go version to execute.
    let version = {
        // 1a. Look for a version pinned in the project.
        let mut current_dir = env::current_dir()?;
        let mut pinned_version = None;

        loop {
            let pin_file_path = current_dir.join(".golta.json");
            if pin_file_path.exists() {
                let content = fs::read_to_string(pin_file_path)?;
                let json: serde_json::Value = serde_json::from_str(&content)?;
                if let Some(go_ver) = json.get("go").and_then(|v| v.as_str()) {
                    pinned_version = Some(go_ver.to_string());
                }
                break; // If .golta.json is found, stop searching.
            }

            if !current_dir.pop() {
                break; // 親ディレクトリがなければ（ルートまで達したら）終了
            }
        }

        // 1b. If not pinned, read the global default version.
        pinned_version.unwrap_or_else(|| {
            let home = home::home_dir()
                .ok_or("Could not find home directory")
                .unwrap();
            let default_file = home.join(".golta").join("state").join("default.txt");
            fs::read_to_string(default_file)
                .unwrap_or_default()
                .trim()
                .to_string()
        })
    };

    // If the version could not be determined, return an error.
    if version.is_empty() {
        return Err("No Go version is set. Use `golta pin go@<version>` in your project, or `golta default go@<version>` globally.".into());
    }

    // 2. Construct the path to the actual Go binary.
    let home = home::home_dir().ok_or("Could not find home directory")?;
    let go_executable_name = if cfg!(windows) { "go.exe" } else { "go" };
    let real_go_path: PathBuf = home
        .join(".golta")
        .join("versions")
        .join(version.trim_start_matches("go@")) // Remove the "go@" prefix.
        .join("bin")
        .join(go_executable_name);

    // Get arguments.
    let args: Vec<String> = env::args_os()
        .skip(1)
        .map(|s| s.into_string().unwrap())
        .collect();

    // 3. Execute the actual Go binary.
    let status = Command::new(real_go_path).args(args).status()?;

    // Return the exit code of the child process directly.
    // Also consider termination by a signal (Unix-like systems) and return 1 by default.
    exit(status.code().unwrap_or(1));
}
