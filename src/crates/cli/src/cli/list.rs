use crate::shared::local_versions::get_installed_versions;
use crate::shared::pinned_version::find_pinned_go_version;
use regex::Regex;
use semver::Version;
use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

pub fn run(tool_opt: Option<String>) {
    let tool = tool_opt.unwrap_or_else(|| "go".to_string());
    let ctx = FsListContext;
    let mut out = std::io::stdout();
    if let Err(e) = list_versions(&ctx, &tool, &mut out) {
        eprintln!("Error: {}", e);
    }
}

fn list_versions(
    ctx: &impl ListContext,
    tool: &str,
    out: &mut dyn Write,
) -> Result<(), Box<dyn Error>> {
    let default_version = ctx.read_default_version(tool);

    // Pinning is currently only supported for "go"
    let pinned_version = if tool == "go" {
        ctx.pinned_go_version()?
    } else {
        None
    };
    let active_version = pinned_version.clone().or_else(|| default_version.clone());

    writeln!(out, "Installed {} versions:", tool)?;

    let installed_strings = ctx.installed_versions(tool)?;
    let mut sortable_versions: Vec<(Option<Version>, String)> = installed_strings
        .iter()
        .map(|s| {
            let normalized = normalize_version(s);
            (Version::parse(&normalized).ok(), s.clone())
        })
        .collect();

    sortable_versions.sort_by(|(v1, s1), (v2, s2)| match (v1, v2) {
        (Some(v1), Some(v2)) => v2.cmp(v1),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => s1.cmp(s2),
    });

    if sortable_versions.is_empty() {
        writeln!(out, "  No {} versions installed", tool)?;
        return Ok(());
    }

    for (_version_obj, original_version_str) in sortable_versions {
        let version = original_version_str;
        let mut tags = Vec::new();
        let is_active = active_version.as_deref() == Some(version.as_str());
        let is_default = default_version.as_deref() == Some(&version);
        let is_pinned = pinned_version.as_deref() == Some(&version);

        if is_default {
            tags.push("default");
        }
        if is_pinned {
            tags.push("pinned");
        }

        let prefix = if is_active { "*" } else { " " };
        let tag_str = if tags.is_empty() {
            String::new()
        } else {
            format!(" ({})", tags.join(", "))
        };

        writeln!(out, "{} {}{}", prefix, version, tag_str)?;
    }

    Ok(())
}

trait ListContext {
    fn home_dir(&self) -> Option<PathBuf>;
    fn read_default_version(&self, tool: &str) -> Option<String>;
    fn pinned_go_version(&self) -> Result<Option<String>, Box<dyn Error>>;
    fn installed_versions(&self, tool: &str) -> Result<Vec<String>, Box<dyn Error>>;
}

struct FsListContext;

impl ListContext for FsListContext {
    fn home_dir(&self) -> Option<PathBuf> {
        home::home_dir()
    }

    fn read_default_version(&self, tool: &str) -> Option<String> {
        let home = self.home_dir()?;
        let state_dir = home.join(".golta").join("state");
        let path = if tool == "go" {
            state_dir.join("default.txt")
        } else {
            state_dir.join(format!("{}.default", tool))
        };
        std::fs::read_to_string(path)
            .ok()
            .map(|s| s.trim().to_string())
    }

    fn pinned_go_version(&self) -> Result<Option<String>, Box<dyn Error>> {
        Ok(find_pinned_go_version()?.map(|(v, _)| v))
    }

    fn installed_versions(&self, tool: &str) -> Result<Vec<String>, Box<dyn Error>> {
        if tool == "go" {
            get_installed_versions()
        } else {
            let home = self.home_dir().ok_or("Could not find home directory")?;
            let tool_dir = home.join(".golta").join("versions").join(tool);
            if !tool_dir.exists() {
                return Ok(Vec::new());
            }

            let mut versions = Vec::new();
            for entry in fs::read_dir(tool_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    if let Ok(name) = entry.file_name().into_string() {
                        versions.push(name);
                    }
                }
            }
            Ok(versions)
        }
    }
}

// Helper function to normalize version strings to a semver-compatible format
fn normalize_version(version_str: &str) -> String {
    // Strip 'v' prefix if present (common in tools like air, dlv)
    let version_str = version_str.strip_prefix('v').unwrap_or(version_str);

    // Regex to match "MAJOR.MINORrcX" or "MAJOR.MINORbetaX"
    // e.g., "1.3rc1" -> "1.3.0-rc1"
    // e.g., "1.4beta1" -> "1.4.0-beta1"
    let re = Regex::new(r"^(\d+\.\d+)(rc|beta)(\d+)$").unwrap();
    if let Some(caps) = re.captures(version_str) {
        let major_minor = caps.get(1).unwrap().as_str();
        let pre_type = caps.get(2).unwrap().as_str();
        let pre_num = caps.get(3).unwrap().as_str();
        format!("{}.0-{}{}", major_minor, pre_type, pre_num)
    } else {
        version_str.to_string() // Return original if no special Go pre-release format
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCtx {
        home: Option<PathBuf>,
        default: Option<String>,
        pinned: Option<String>,
        installed: Vec<String>,
    }

    impl Default for MockCtx {
        fn default() -> Self {
            Self {
                home: Some(PathBuf::from("/home/user")),
                default: None,
                pinned: None,
                installed: Vec::new(),
            }
        }
    }

    impl ListContext for MockCtx {
        fn home_dir(&self) -> Option<PathBuf> {
            self.home.clone()
        }

        fn read_default_version(&self, _tool: &str) -> Option<String> {
            self.default.clone()
        }

        fn pinned_go_version(&self) -> Result<Option<String>, Box<dyn Error>> {
            Ok(self.pinned.clone())
        }

        fn installed_versions(&self, _tool: &str) -> Result<Vec<String>, Box<dyn Error>> {
            Ok(self.installed.clone())
        }
    }

    #[test]
    fn shows_message_when_no_installed_versions() {
        let ctx = MockCtx::default();
        let mut out: Vec<u8> = Vec::new();

        list_versions(&ctx, "go", &mut out).unwrap();

        let output = String::from_utf8(out).unwrap();
        assert!(output.contains("Installed go versions:"));
        assert!(output.contains("No go versions installed"));
    }

    #[test]
    fn marks_default_and_active_versions() {
        let ctx = MockCtx {
            default: Some("1.20.0".to_string()),
            installed: vec![
                "1.20.0".to_string(),
                "1.21.0".to_string(),
                "1.3rc1".to_string(),
            ],
            ..MockCtx::default()
        };
        let mut out: Vec<u8> = Vec::new();

        list_versions(&ctx, "go", &mut out).unwrap();

        let output = String::from_utf8(out).unwrap();
        assert!(output.contains("* 1.20.0 (default)"));
        assert!(output.contains("  1.21.0"));
        assert!(
            output.contains("1.3rc1"),
            "pre-release versions should still render"
        );
    }

    #[test]
    fn marks_pinned_version_as_active() {
        let ctx = MockCtx {
            pinned: Some("1.21.2".to_string()),
            installed: vec!["1.21.2".to_string(), "1.20.0".to_string()],
            ..MockCtx::default()
        };
        let mut out: Vec<u8> = Vec::new();

        list_versions(&ctx, "go", &mut out).unwrap();

        let output = String::from_utf8(out).unwrap();
        assert!(
            output.contains("* 1.21.2 (pinned)"),
            "pinned version should be active and tagged"
        );
        assert!(
            !output.contains("* 1.20.0"),
            "only pinned should be active when both exist"
        );
    }

    #[test]
    fn normalizes_v_prefix() {
        assert_eq!(normalize_version("v1.2.3"), "1.2.3");
        assert_eq!(normalize_version("1.2.3"), "1.2.3");
    }
}
