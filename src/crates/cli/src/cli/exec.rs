use crate::shared::active_version::find_active_go_version;
use std::error::Error;
use std::process::Command;

pub fn run(tool: String, args: Vec<String>) {
    // If an error occurs, print a message and exit.
    if let Err(e) = exec_go(&tool, &args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn exec_go(tool: &str, args: &[String]) -> Result<(), Box<dyn Error>> {
    if tool != "go" {
        return Err("Only `go` is supported for exec currently.".into());
    }

    let version_str = find_active_go_version()?
        .ok_or("No Go version is active. Use `golta pin` or `golta default`.")?;

    if version_str.is_empty() {
        return Err("No Go version is active. Use `golta pin` or `golta default`.".into());
    }

    let go_executable_name = if cfg!(windows) { "go.exe" } else { "go" };
    let home = home::home_dir().ok_or("Could not find home directory")?;
    let go_path = home
        .join(".golta")
        .join("versions")
        .join(version_str.trim_start_matches("go@"))
        .join("bin")
        .join(go_executable_name);

    let status = Command::new(go_path).args(args).status()?; // Handle I/O errors with the `?` operator.

    // Use the exit code of the child process as our own.
    std::process::exit(status.code().unwrap_or(1));
}
