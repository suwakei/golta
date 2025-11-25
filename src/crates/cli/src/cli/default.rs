use crate::DefaultCommand;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

pub fn run(cmd: DefaultCommand) {
    if let Err(e) = handle_default(cmd) {
        eprintln!("Error: {}", e);
    }
}

fn handle_default(cmd: DefaultCommand) -> Result<(), Box<dyn Error>> {
    let state_dir = get_state_dir()?;
    let default_file = state_dir.join("default.txt");

    match cmd.command {
        Some(crate::DefaultCommands::Clear) => {
            if default_file.exists() {
                fs::remove_file(default_file)?;
                println!("Cleared global default Go version.");
            } else {
                println!("No global default Go version is set.");
            }
        }
        None => {
            let tool = cmd.tool.unwrap(); // `required_unless_present` ensures this is Some
            if !tool.starts_with("go@") {
                return Err(
                    "Only Go default version is supported currently. Use format 'go@<version>'."
                        .into(),
                );
            }
            let version = tool.trim_start_matches("go@");

            // Check if the version is installed before setting it as default
            let installed_versions = get_installed_versions()?;

            let version_exists = installed_versions.iter().any(|v| v == version);

            if !version_exists {
                return Err(format!("Go version {} is not installed. Please install it first with `golta install go@{}`.", version, version).into());
            }

            fs::create_dir_all(&state_dir)?;
            fs::write(&default_file, version)?;

            println!("Set Go default version to {}", version);
        }
    }

    Ok(())
}

fn get_state_dir() -> Result<PathBuf, Box<dyn Error>> {
    let home = home::home_dir().ok_or("Could not find home directory")?;
    let state_dir = home.join(".golta").join("state");
    Ok(state_dir)
}

fn get_installed_versions() -> Result<Vec<String>, Box<dyn Error>> {
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
