use std::error::Error;
use std::fs;

pub fn run() {
    if let Err(e) = list_go_versions() {
        eprintln!("Error: {}", e);
    }
}

fn list_go_versions() -> Result<(), Box<dyn Error>> {
    // Use `home::home_dir` to safely get the home directory in a cross-platform way
    let home = home::home_dir().ok_or("Could not find home directory")?;
    let golta_dir = home.join(".golta");
    let versions_dir = golta_dir.join("versions");
    let default_file = golta_dir.join("state").join("default.txt");

    // Read the default version. Do not error if the file does not exist.
    let default_version = fs::read_to_string(default_file).ok();
    // `trim()` to remove surrounding whitespace, `trim_start_matches` to remove the prefix
    let default_version_name = default_version
        .as_deref()
        .map(|s| s.trim().trim_start_matches("go@"));

    println!("Installed Go versions:");

    if versions_dir.exists() {
        let mut entries = Vec::new();
        for entry in fs::read_dir(versions_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    entries.push(name.to_string());
                }
            }
        }

        entries.sort();

        for version_name in entries {
            if Some(version_name.as_str()) == default_version_name {
                println!("* {} (default)", version_name);
            } else {
                println!("  {}", version_name);
            }
        }
    } else {
        println!("  No Go versions installed");
    }

    Ok(())
}
