use crate::shared::pinned_version::find_pinned_go_version;
use std::error::Error;
use std::fs;

pub fn run(tool: String) {
    if let Err(e) = uninstall_go(&tool) {
        eprintln!("Error: {}", e);
    }
}
fn uninstall_go(tool: &str) -> Result<(), Box<dyn Error>> {
    if !tool.starts_with("go@") {
        return Err("Only Go uninstall is supported currently. Use format 'go@<version>'.".into());
    }

    let version = tool.trim_start_matches("go@");

    // Use `home::home_dir` to safely get the home directory in a cross-platform way
    let home = home::home_dir().ok_or("Could not find home directory")?;
    let golta_dir = home.join(".golta");
    let version_dir = golta_dir.join("versions").join(version);

    if !version_dir.exists() {
        return Err(format!("Go {} is not installed.", version).into());
    }

    // If it's the same as the default version, show a warning and clear the default.
    let default_file = golta_dir.join("state").join("default.txt");
    if default_file.exists() {
        if let Ok(default_version) = fs::read_to_string(&default_file) {
            if default_version.trim() == version {
                println!("Warning: uninstalling the default Go version ({}). The global default will be cleared.", version);
                fs::remove_file(&default_file)?;
            }
        }
    }

    // Check if the version is pinned in the current project structure and show a warning.
    if let Some((pinned_version, pin_file_path)) = find_pinned_go_version()? {
        if pinned_version == version {
            println!(
                "Warning: uninstalling version {}, which is pinned in {}",
                version,
                pin_file_path.display()
            );
        }
    }

    // Remove the version
    fs::remove_dir_all(&version_dir)?;
    println!("Go {} has been uninstalled.", version);

    Ok(())
}
