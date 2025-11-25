use std::error::Error;
use std::fs;

/// Returns a list of locally installed Go version strings.
/// It reads the `~/.golta/versions` directory and returns the names of the subdirectories.
pub fn get_installed_versions() -> Result<Vec<String>, Box<dyn Error>> {
    let home = home::home_dir().ok_or("Could not find home directory")?;
    let versions_dir = home.join(".golta").join("versions");

    if !versions_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(versions_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                entries.push(name.to_string());
            }
        }
    }
    Ok(entries)
}
