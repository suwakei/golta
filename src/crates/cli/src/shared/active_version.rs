use crate::shared::pinned_version::find_pinned_go_version;
use std::error::Error;
use std::fs;

/// Finds the active Go version by checking for a pinned version first, then a global default.
pub fn find_active_go_version() -> Result<Option<String>, Box<dyn Error>> {
    // 1a. Look for a version pinned to the project.
    if let Some((pinned_version, _)) = find_pinned_go_version()? {
        return Ok(Some(pinned_version));
    }

    // 1b. If not pinned, read the global default version
    let home = match home::home_dir() {
        Some(path) => path,
        None => return Ok(None), // No home directory, so no default.
    };
    let default_file = home.join(".golta").join("state").join("default.txt");

    if default_file.exists() {
        let default_version = fs::read_to_string(default_file)?;
        let trimmed_version = default_version.trim();
        if !trimmed_version.is_empty() {
            return Ok(Some(trimmed_version.to_string()));
        }
    }

    Ok(None)
}
