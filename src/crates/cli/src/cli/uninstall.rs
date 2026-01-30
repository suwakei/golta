use crate::shared::pinned_version::find_pinned_go_version;
use indicatif::{ProgressBar, ProgressStyle};
use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub fn run(tool_arg: String) {
    let mut stdout = io::stdout();
    let home = match home::home_dir() {
        Some(path) => path,
        None => {
            eprintln!("Error: Could not find home directory");
            return;
        }
    };

    if let Err(e) = uninstall_tool(&tool_arg, &home, find_pinned_go_version, &mut stdout) {
        eprintln!("Error: {}", e);
    }
}

fn uninstall_tool<W, F>(
    tool_arg: &str,
    home: &Path,
    find_pinned: F,
    writer: &mut W,
) -> Result<(), Box<dyn Error>>
where
    W: Write,
    F: Fn() -> Result<Option<(String, PathBuf)>, Box<dyn Error>>,
{
    let (tool, version) = parse_tool_version(tool_arg)?;
    let (version_dir, default_file) = build_paths(home, &tool, &version);

    if !version_dir.exists() {
        return Err(format!("{} {} is not installed.", tool, version).into());
    }

    clear_default_if_matches(&tool, &version, &default_file, writer)?;

    if tool == "go" {
        warn_if_pinned(&version, find_pinned, writer)?;
    }

    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::with_template("{spinner:.green} {msg}")?);
    pb.set_message(format!("Uninstalling {} {}...", tool, version));
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    fs::remove_dir_all(&version_dir)?;
    pb.finish_and_clear();
    writeln!(writer, "{} {} has been uninstalled.", tool, version)?;

    Ok(())
}

fn parse_tool_version(input: &str) -> Result<(String, String), Box<dyn Error>> {
    if let Some((tool, version)) = input.split_once('@') {
        if !tool.is_empty() && !version.is_empty() {
            return Ok((tool.to_string(), version.to_string()));
        }
    }
    Err("Invalid format. Use <tool>@<version>.".into())
}

fn build_paths(home: &Path, tool: &str, version: &str) -> (PathBuf, PathBuf) {
    let golta_dir = home.join(".golta");
    let version_dir = if tool == "go" {
        golta_dir.join("versions").join(version)
    } else {
        golta_dir.join("versions").join(tool).join(version)
    };

    let default_file = if tool == "go" {
        golta_dir.join("state").join("default.txt")
    } else {
        golta_dir.join("state").join(format!("{}.default", tool))
    };
    (version_dir, default_file)
}

fn clear_default_if_matches<W: Write>(
    tool: &str,
    version: &str,
    default_file: &Path,
    writer: &mut W,
) -> Result<(), Box<dyn Error>> {
    if default_file.exists() {
        if let Ok(default_version) = fs::read_to_string(default_file) {
            if default_version.trim() == version {
                writeln!(
                    writer,
                    "Warning: uninstalling the default {} version ({}). The global default will be cleared.",
                    tool, version
                )?;
                fs::remove_file(default_file)?;
            }
        }
    }

    Ok(())
}

fn warn_if_pinned<W, F>(version: &str, find_pinned: F, writer: &mut W) -> Result<(), Box<dyn Error>>
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
        let (version_dir, default_file) = build_paths(&home, "go", version);

        fs::create_dir_all(&version_dir).unwrap();
        fs::create_dir_all(default_file.parent().unwrap()).unwrap();
        fs::write(&default_file, version).unwrap();

        let mut buffer = Vec::new();
        uninstall_tool(&format!("go@{}", version), &home, || Ok(None), &mut buffer).unwrap();

        assert!(!version_dir.exists());
        assert!(!default_file.exists());

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("Warning: uninstalling the default go version"));
        assert!(output.contains("Go 1.22.3 has been uninstalled."));

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn warns_when_version_is_pinned() {
        let home = temp_home();
        let version = "1.22.4";
        let (version_dir, _) = build_paths(&home, "go", version);
        fs::create_dir_all(&version_dir).unwrap();

        let pin_path = PathBuf::from("/project/.golta-version");
        let mut buffer = Vec::new();
        uninstall_tool(
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

    #[test]
    fn uninstalls_other_tool() {
        let home = temp_home();
        let tool = "air";
        let version = "v1.51.0";
        let (version_dir, default_file) = build_paths(&home, tool, version);

        fs::create_dir_all(&version_dir).unwrap();
        fs::create_dir_all(default_file.parent().unwrap()).unwrap();

        let mut buffer = Vec::new();
        uninstall_tool("air@v1.51.0", &home, || Ok(None), &mut buffer).unwrap();

        assert!(!version_dir.exists());

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("air v1.51.0 has been uninstalled."));
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
