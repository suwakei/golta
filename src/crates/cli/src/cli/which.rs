use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

pub fn run(tool: String) {
    if let Err(e) = which_go(&tool) {
        eprintln!("Error: {}", e);
    }
}

fn which_go(tool: &str) -> Result<(), Box<dyn Error>> {
    if tool != "go" {
        return Err("Only `go` is supported for which currently.".into());
    }

    // 1. Determine the Go version to use (same logic as the shim)
    let version = {
        // 1a. Look for a version pinned to the project
        let mut current_dir = env::current_dir()?;
        let mut pinned_version = None;

        loop {
            let pin_file_path = current_dir.join(".golta.json");
            if pin_file_path.exists() {
                let content = fs::read_to_string(pin_file_path)?;
                // serde_json::from_str returns a Result, so use `?` for error handling
                let json: serde_json::Value = serde_json::from_str(&content)?;
                if let Some(go_ver) = json.get("go").and_then(|v| v.as_str()) {
                    pinned_version = Some(go_ver.to_string());
                }
                break; // If .golta.json is found, stop searching
            }

            if !current_dir.pop() {
                break; // If there's no parent directory (reached the root), stop
            }
        }

        // 1b. If not pinned, read the global default version
        pinned_version.unwrap_or_else(|| {
            // The .unwrap() here could panic, so return a Result instead
            home::home_dir()
                .ok_or_else(|| "Could not find home directory".to_string())
                .and_then(|home| {
                    let default_file = home.join(".golta").join("state").join("default.txt");
                    fs::read_to_string(default_file).map_err(|e| e.to_string())
                })
                .unwrap_or_default() // Return an empty string on error
                .trim()
                .to_string()
        })
    };

    if version.is_empty() {
        return Err("No Go version is active. Use `golta pin` or `golta default`.".into());
    }

    // 2. Construct the path to the actual Go binary
    let home = home::home_dir().ok_or("Could not find home directory")?;
    let go_executable_name = if cfg!(windows) { "go.exe" } else { "go" };
    let go_path: PathBuf = home
        .join(".golta")
        .join("versions")
        .join(version.trim_start_matches("go@"))
        .join("bin")
        .join(go_executable_name);

    println!("{}", go_path.display());
    Ok(())
}
