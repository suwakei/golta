use crate::shared::active_version::find_active_go_version;
use std::error::Error;
use std::path::{Path, PathBuf};

pub fn run(tool: String) {
    match which_go(&tool) {
        Ok(path) => println!("{}", path.display()),
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn which_go(tool: &str) -> Result<PathBuf, Box<dyn Error>> {
    if tool != "go" {
        return Err("Only `go` is supported for which currently.".into());
    }

    let version = find_active_go_version()?
        .ok_or("No Go version is active. Use `golta pin` or `golta default`.")?;

    if version.is_empty() {
        return Err("No Go version is active. Use `golta pin` or `golta default`.".into());
    }

    let home = home::home_dir().ok_or("Could not find home directory")?;
    Ok(resolve_go_path(&home, &version))
}

fn resolve_go_path(home: &Path, version: &str) -> PathBuf {
    let go_executable_name = if cfg!(windows) { "go.exe" } else { "go" };

    home.join(".golta")
        .join("versions")
        .join(version.trim_start_matches("go@"))
        .join("go")
        .join("bin")
        .join(go_executable_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_go_path_with_prefix_removed() {
        let home = PathBuf::from("/tmp/home");
        let path = resolve_go_path(&home, "go@1.22.3");

        let expected = home
            .join(".golta")
            .join("versions")
            .join("1.22.3")
            .join("go")
            .join("bin")
            .join(if cfg!(windows) { "go.exe" } else { "go" });

        assert_eq!(path, expected);
    }

    #[test]
    fn builds_go_path_without_prefix() {
        let home = PathBuf::from("/tmp/home");
        let path = resolve_go_path(&home, "1.22.3");

        let expected = home
            .join(".golta")
            .join("versions")
            .join("1.22.3")
            .join("go")
            .join("bin")
            .join(if cfg!(windows) { "go.exe" } else { "go" });

        assert_eq!(path, expected);
    }
}
