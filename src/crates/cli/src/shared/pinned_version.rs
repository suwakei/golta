use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

/// Finds a pinned Go version (`.golta.json`) by traversing up from the current directory.
/// Returns the version and the path to the pin file if found.
pub fn find_pinned_go_version() -> Result<Option<(String, PathBuf)>, Box<dyn Error>> {
    let mut current_dir = env::current_dir()?;
    loop {
        let pin_file_path = current_dir.join(".golta.json");
        if pin_file_path.exists() {
            let content = fs::read_to_string(&pin_file_path)?;
            let json: serde_json::Value = serde_json::from_str(&content)?;
            if let Some(go_ver) = json.get("go").and_then(|v| v.as_str()) {
                return Ok(Some((go_ver.to_string(), pin_file_path)));
            }
        }

        if !current_dir.pop() {
            return Ok(None); // Reached the root
        }
    }
}
