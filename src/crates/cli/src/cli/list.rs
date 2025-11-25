use crate::shared::local_versions::get_installed_versions;
use crate::shared::pinned_version::find_pinned_go_version;
use regex::Regex;
use semver::Version;
use std::error::Error;
use std::fs;

pub fn run() {
    if let Err(e) = list_go() {
        eprintln!("Error: {}", e);
    }
}

fn list_go() -> Result<(), Box<dyn Error>> {
    let home = home::home_dir().ok_or("Could not find home directory")?;
    let golta_dir = home.join(".golta");
    let default_file = golta_dir.join("state").join("default.txt");

    let default_version = fs::read_to_string(default_file)
        .ok()
        .map(|s| s.trim().to_string());

    let pinned_info = find_pinned_go_version()?;
    let pinned_version = pinned_info.as_ref().map(|(v, _)| v.clone());

    let active_version = pinned_version.clone().or_else(|| default_version.clone());

    println!("Installed Go versions:");

    let installed_strings = get_installed_versions()?;
    let mut sortable_versions: Vec<(Version, String)> = installed_strings
        .iter()
        .filter_map(|s| {
            let normalized = normalize_go_version_for_semver(s);
            Version::parse(&normalized).ok().map(|v| (v, s.clone())) // Store parsed Version and original string
        })
        .collect();

    sortable_versions.sort_by(|(v1, _), (v2, _)| v1.cmp(v2));

    if sortable_versions.is_empty() {
        println!("  No Go versions installed");
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

        println!("{} {}{}", prefix, version, tag_str);
    }

    Ok(())
}

// Helper function to normalize Go version strings to a semver-compatible format
// This handles Go-specific pre-release formats like "1.3rc1" by converting them
// to "1.3.0-rc1" which `semver::Version::parse` can understand.
fn normalize_go_version_for_semver(version_str: &str) -> String {
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
