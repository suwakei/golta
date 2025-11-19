use std::error::Error;
use std::process::Command;

pub fn run(tool: String, args: Vec<String>) {
    // If an error occurs, print a message and exit.
    if let Err(e) = run_go(&tool, &args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run_go(tool: &str, args: &[String]) -> Result<(), Box<dyn Error>> {
    if !tool.starts_with("go@") {
        return Err("Only Go run is supported currently. Use format 'go@<version>'.".into());
    }

    let version = tool.trim_start_matches("go@");

    // Use `home::home_dir` to safely get the home directory in a cross-platform way
    let home = home::home_dir().ok_or("Could not find home directory")?;

    let go_executable_name = if cfg!(windows) { "go.exe" } else { "go" };
    let go_path = home
        .join(".golta")
        .join("versions")
        .join(version)
        .join("bin")
        .join(go_executable_name);

    let status = Command::new(go_path).args(args).status()?; // Handle I/O errors with the `?` operator.

    // Use the exit code of the child process as our own.
    std::process::exit(status.code().unwrap_or(1));
}
