use crate::shared::pinned_version::find_pinned_go_version;
use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub fn run(tool: String) {
    let mut stdout = io::stdout();
    let home = match home::home_dir() {
        Some(path) => path,
        None => {
            eprintln!("Error: Could not find home directory");
            return;
        }
    };

    if let Err(e) = uninstall_go(&tool, &home, find_pinned_go_version, &mut stdout) {
        eprintln!("Error: {}", e);
    }
}

fn uninstall_go<W, F>(
    tool: &str,
    home: &Path,
    find_pinned: F,
    writer: &mut W,
) -> Result<(), Box<dyn Error>>
where
    W: Write,
    F: Fn() -> Result<Option<(String, PathBuf)>, Box<dyn Error>>,
{
    if !tool.starts_with("go@") {
        return Err("Only Go uninstall is supported currently. Use format 'go@<version>'.".into());
    }

    let version = tool.trim_start_matches("go@");
    let (version_dir, default_file) = build_paths(home, version);

    if !version_dir.exists() {
        return Err(format!("Go {} is not installed.", version).into());
    }

    clear_default_if_matches(version, &default_file, writer)?;
    warn_if_pinned(version, find_pinned, writer)?;

    fs::remove_dir_all(&version_dir)?;
    writeln!(writer, "Go {} has been uninstalled.", version)?;

    Ok(())
}

fn build_paths(home: &Path, version: &str) -> (PathBuf, PathBuf) {
    let golta_dir = home.join(".golta");
    let version_dir = golta_dir.join("versions").join(version);
    let default_file = golta_dir.join("state").join("default.txt");
    (version_dir, default_file)
}

fn clear_default_if_matches<W: Write>(
    version: &str,
    default_file: &Path,
    writer: &mut W,
) -> Result<(), Box<dyn Error>> {
    if default_file.exists() {
        if let Ok(default_version) = fs::read_to_string(default_file) {
            if default_version.trim() == version {
                writeln!(
                    writer,
                    "Warning: uninstalling the default Go version ({}). The global default will be cleared.",
                    version
                )?;
                fs::remove_file(default_file)?;
            }
        }
    }

    Ok(())
}

fn warn_if_pinned<W, F>(
    version: &str,
    find_pinned: F,
    writer: &mut W,
) -> Result<(), Box<dyn Error>>
where
    W: Write,
    F: Fn() -> Result<Option<(String, PathBuf)>, Box<dyn Error>>,
{
    if let Some((pinned_version, pin_file_path)) = find_pinned()? {
        if pinned_version == version {
            writeln!(
                writer,
                "Warning: uninstalling version {}, which is pinned in {}",
                version,
                pin_file_path.display()
            )?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clears_default_and_removes_version_dir() {
        let home = temp_home();
        let version = "1.22.3";
        let (version_dir, default_file) = build_paths(&home, version);

        fs::create_dir_all(&version_dir).unwrap();
        fs::create_dir_all(default_file.parent().unwrap()).unwrap();
        fs::write(&default_file, version).unwrap();

        let mut buffer = Vec::new();
        uninstall_go(&format!("go@{}", version), &home, || Ok(None), &mut buffer).unwrap();

        assert!(!version_dir.exists());
        assert!(!default_file.exists());

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("Warning: uninstalling the default Go version"));
        assert!(output.contains("Go 1.22.3 has been uninstalled."));

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn warns_when_version_is_pinned() {
        let home = temp_home();
        let version = "1.22.4";
        let (version_dir, _) = build_paths(&home, version);
        fs::create_dir_all(&version_dir).unwrap();

        let pin_path = PathBuf::from("/project/.golta-version");
        let mut buffer = Vec::new();
        uninstall_go(
            &format!("go@{}", version),
            &home,
            || Ok(Some((version.to_string(), pin_path.clone()))),
            &mut buffer,
        )
        .unwrap();

        assert!(!version_dir.exists());

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains(pin_path.to_string_lossy().as_ref()));

        fs::remove_dir_all(home).unwrap();
    }

    fn temp_home() -> PathBuf {
        let mut path = std::env::temp_dir();
        let unique = format!(
            "golta_uninstall_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        path.push(unique);
        path
    }
}
