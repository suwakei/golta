use crate::shared::local_versions::get_installed_versions;
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::fs::write;

#[derive(Serialize, Deserialize)]
struct PinFile {
    go: String,
}

pub fn run(tool: String, _update_go_mod: bool) {
    if let Err(e) = pin_go_version(&tool) {
        eprintln!("Error: {}", e);
    }
}

fn pin_go_version(tool: &str) -> Result<(), Box<dyn Error>> {
    if !tool.starts_with("go@") {
        return Err("Invalid format. Use `golta pin go@<version>`.".into());
    }

    let version = tool.trim_start_matches("go@");

    // Check if the version is installed before pinning
    let installed_versions = get_installed_versions()?;
    if !installed_versions.iter().any(|v| v == version) {
        return Err(format!(
            "Go version '{}' is not installed. Please install it first with `golta install go@{}`.",
            version, version
        )
        .into());
    }

    let project_dir = env::current_dir()?;
    let pin_file = project_dir.join(".golta.json");

    let pin = PinFile {
        go: version.to_string(),
    };
    write(&pin_file, serde_json::to_string_pretty(&pin)?)?;

    println!("Pinned Go version {} to {}", version, pin_file.display());

    Ok(())
}
