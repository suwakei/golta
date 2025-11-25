use crate::shared::pinned_version::find_pinned_go_version;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

/// Finds the active Go version by checking for a pinned version first, then a global default.
pub fn find_active_go_version() -> Result<Option<String>, Box<dyn Error>> {
    let fs_finder = FsVersionFinder::new();
    find_active_go_version_logic(&fs_finder)
}

/// The core logic for finding the active Go version, decoupled from the filesystem.
fn find_active_go_version_logic(
    finder: &impl VersionProvider,
) -> Result<Option<String>, Box<dyn Error>> {
    // 1. Look for a version pinned to the project.
    if let Some(pinned_version) = finder.find_pinned_version()? {
        return Ok(Some(pinned_version));
    }

    // 2. If not pinned, look for the global default version.
    if let Some(default_version) = finder.find_default_version()? {
        return Ok(Some(default_version));
    }

    Ok(None)
}

/// `VersionProvider` abstracts the mechanism for finding pinned and default versions.
/// This allows for easy mocking during tests.
trait VersionProvider {
    fn find_pinned_version(&self) -> Result<Option<String>, Box<dyn Error>>;
    fn find_default_version(&self) -> Result<Option<String>, Box<dyn Error>>;
}

/// Filesystem-based implementation of `VersionProvider`.
struct FsVersionFinder {
    home_dir: Option<PathBuf>,
}

impl FsVersionFinder {
    fn new() -> Self {
        Self {
            home_dir: home::home_dir(),
        }
    }
}

impl VersionProvider for FsVersionFinder {
    fn find_pinned_version(&self) -> Result<Option<String>, Box<dyn Error>> {
        if let Some((pinned_version, _)) = find_pinned_go_version()? {
            Ok(Some(pinned_version))
        } else {
            Ok(None)
        }
    }

    fn find_default_version(&self) -> Result<Option<String>, Box<dyn Error>> {
        let home = match &self.home_dir {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockVersionProvider {
        pinned: Result<Option<String>, &'static str>,
        default: Result<Option<String>, &'static str>,
    }

    impl VersionProvider for MockVersionProvider {
        fn find_pinned_version(&self) -> Result<Option<String>, Box<dyn Error>> {
            self.pinned.clone().map_err(|e| e.into())
        }
        fn find_default_version(&self) -> Result<Option<String>, Box<dyn Error>> {
            self.default.clone().map_err(|e| e.into())
        }
    }

    #[test]
    fn test_prefers_pinned_version() {
        let provider = MockVersionProvider {
            pinned: Ok(Some("1.18.0".to_string())),
            default: Ok(Some("1.17.0".to_string())),
        };
        assert_eq!(
            find_active_go_version_logic(&provider).unwrap(),
            Some("1.18.0".to_string())
        );
    }

    #[test]
    fn test_falls_back_to_default_version() {
        let provider = MockVersionProvider {
            pinned: Ok(None),
            default: Ok(Some("1.17.0".to_string())),
        };
        assert_eq!(
            find_active_go_version_logic(&provider).unwrap(),
            Some("1.17.0".to_string())
        );
    }

    #[test]
    fn test_returns_none_when_no_version_is_set() {
        let provider = MockVersionProvider {
            pinned: Ok(None),
            default: Ok(None),
        };
        assert_eq!(find_active_go_version_logic(&provider).unwrap(), None);
    }
}
