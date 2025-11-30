use crate::shared::local_versions::get_installed_versions;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
struct PinFile {
    go: String,
}

pub fn run(tool: String, _update_go_mod: bool) {
    let ctx = FsPinContext;
    let mut out = std::io::stdout();
    if let Err(e) = pin_go_version(&ctx, &tool, &mut out) {
        eprintln!("Error: {}", e);
    }
}

fn pin_go_version(
    ctx: &impl PinContext,
    tool: &str,
    out: &mut dyn Write,
) -> Result<(), Box<dyn Error>> {
    if !tool.starts_with("go@") {
        return Err("Invalid format. Use `golta pin go@<version>`.".into());
    }

    let version = tool.trim_start_matches("go@");

    // Check if the version is installed before pinning
    let installed_versions = ctx.installed_versions()?;
    if !installed_versions.iter().any(|v| v == version) {
        return Err(format!(
            "Go version '{}' is not installed. Please install it first with `golta install go@{}`.",
            version, version
        )
        .into());
    }

    let project_dir = ctx.current_dir()?;
    let pin_file = project_dir.join(".golta.json");

    let pin = PinFile {
        go: version.to_string(),
    };
    let contents = serde_json::to_string_pretty(&pin)?;
    ctx.write_pin_file(&pin_file, &contents)?;

    writeln!(
        out,
        "Pinned Go version {} to {}",
        version,
        pin_file.display()
    )?;

    Ok(())
}

trait PinContext {
    fn installed_versions(&self) -> Result<Vec<String>, Box<dyn Error>>;
    fn current_dir(&self) -> Result<PathBuf, Box<dyn Error>>;
    fn write_pin_file(&self, path: &Path, contents: &str) -> Result<(), Box<dyn Error>>;
}

struct FsPinContext;

impl PinContext for FsPinContext {
    fn installed_versions(&self) -> Result<Vec<String>, Box<dyn Error>> {
        get_installed_versions()
    }

    fn current_dir(&self) -> Result<PathBuf, Box<dyn Error>> {
        Ok(std::env::current_dir()?)
    }

    fn write_pin_file(&self, path: &Path, contents: &str) -> Result<(), Box<dyn Error>> {
        Ok(std::fs::write(path, contents)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct MockPinContext {
        installed: Vec<String>,
        current_dir: PathBuf,
        written: RefCell<Vec<(PathBuf, String)>>,
        fail_write: bool,
    }

    impl Default for MockPinContext {
        fn default() -> Self {
            Self {
                installed: Vec::new(),
                current_dir: PathBuf::from("/tmp/project"),
                written: RefCell::new(Vec::new()),
                fail_write: false,
            }
        }
    }

    impl PinContext for MockPinContext {
        fn installed_versions(&self) -> Result<Vec<String>, Box<dyn Error>> {
            Ok(self.installed.clone())
        }

        fn current_dir(&self) -> Result<PathBuf, Box<dyn Error>> {
            Ok(self.current_dir.clone())
        }

        fn write_pin_file(&self, path: &Path, contents: &str) -> Result<(), Box<dyn Error>> {
            if self.fail_write {
                return Err("write failed".into());
            }
            self.written
                .borrow_mut()
                .push((path.to_path_buf(), contents.to_string()));
            Ok(())
        }
    }

    #[test]
    fn rejects_invalid_format() {
        let ctx = MockPinContext::default();
        let mut out: Vec<u8> = Vec::new();

        let err = pin_go_version(&ctx, "node@1.0.0", &mut out).unwrap_err();

        assert!(err
            .to_string()
            .contains("Invalid format. Use `golta pin go@<version>`."));
    }

    #[test]
    fn errors_when_version_not_installed() {
        let ctx = MockPinContext::default();
        let mut out: Vec<u8> = Vec::new();

        let err = pin_go_version(&ctx, "go@1.20.0", &mut out).unwrap_err();

        assert!(err
            .to_string()
            .contains("Go version '1.20.0' is not installed"));
    }

    #[test]
    fn writes_pin_file_and_reports_success() {
        let ctx = MockPinContext {
            installed: vec!["1.21.0".to_string()],
            ..MockPinContext::default()
        };
        let mut out: Vec<u8> = Vec::new();

        pin_go_version(&ctx, "go@1.21.0", &mut out).unwrap();

        let written = ctx.written.borrow();
        assert_eq!(written.len(), 1);
        let (path, contents) = &written[0];
        assert_eq!(path, &ctx.current_dir.join(".golta.json"));
        assert!(contents.contains("1.21.0"));

        let output = String::from_utf8(out).unwrap();
        assert!(output.contains("Pinned Go version 1.21.0 to"));
    }
}
