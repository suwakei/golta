use std::error::Error;
use std::fs;
use std::path::Path;

/// Returns a list of locally installed Go version strings.
/// It reads the `~/.golta/versions` directory and returns the names of the subdirectories.
pub fn get_installed_versions() -> Result<Vec<String>, Box<dyn Error>> {
    let home = home::home_dir().ok_or("Could not find home directory")?;
    let versions_dir = home.join(".golta").join("versions");
    get_installed_versions_from_path(&versions_dir)
}

/// Returns a list of Go version strings from a specific directory path.
/// It reads the given directory and returns the names of the subdirectories.
fn get_installed_versions_from_path(versions_dir: &Path) -> Result<Vec<String>, Box<dyn Error>> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_get_installed_versions_empty_dir() {
        // Arrange: Create an empty temporary directory.
        let dir = tempdir().unwrap();
        let versions_dir = dir.path();

        // Act
        let result = get_installed_versions_from_path(versions_dir).unwrap();

        // Assert
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_get_installed_versions_with_subdirs() {
        // Arrange
        let dir = tempdir().unwrap();
        let versions_dir = dir.path();

        // Create mock Go version directories.
        fs::create_dir(versions_dir.join("go1.20.5")).unwrap();
        fs::create_dir(versions_dir.join("go1.21.0")).unwrap();
        // Also create a file to ensure it's ignored.
        fs::write(versions_dir.join("not_a_dir"), b"dummy").unwrap();

        // Act
        let mut versions = get_installed_versions_from_path(versions_dir).unwrap();
        versions.sort();

        // Assert
        assert_eq!(versions, vec!["go1.20.5", "go1.21.0"]);
    }

    #[test]
    fn test_get_installed_versions_dir_not_exist() {
        // Arrange: Specify a non-existent directory.
        let dir = tempdir().unwrap();
        let nonexistent = dir.path().join("no_such_dir");

        // Act
        let result = get_installed_versions_from_path(&nonexistent).unwrap();

        // Assert: Should return an empty vector if the directory doesn't exist.
        assert!(result.is_empty());
    }
}
