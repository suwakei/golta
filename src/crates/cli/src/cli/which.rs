use crate::shared::active_version::find_active_go_version;
use std::error::Error;
use std::path::PathBuf;

pub fn run(tool: String) {
    if let Err(e) = which_go(&tool) {
        eprintln!("Error: {}", e);
    }
}

fn which_go(tool: &str) -> Result<(), Box<dyn Error>> {
    if tool != "go" {
        return Err("Only `go` is supported for which currently.".into());
    }

    // 1. Determine the Go version to use.
    let version = find_active_go_version()?
        .ok_or("No Go version is active. Use `golta pin` or `golta default`.")?;

    if version.is_empty() {
        return Err("No Go version is active. Use `golta pin` or `golta default`.".into());
    }

    // 2. Construct the path to the actual Go binary
    let home = home::home_dir().ok_or("Could not find home directory")?;
    let go_executable_name = if cfg!(windows) { "go.exe" } else { "go" };
    let go_path: PathBuf = home
        .join(".golta")
        .join("versions")
        .join(version.trim_start_matches("go@"))
        .join("bin")
        .join(go_executable_name);

    println!("{}", go_path.display());
    Ok(())
}
