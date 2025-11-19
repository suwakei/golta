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

    // Error if it's the same as the default version
    let default_file = golta_dir.join("state").join("default.txt");
    if let Ok(default_version) = fs::read_to_string(&default_file) {
        if default_version.trim().trim_start_matches("go@") == version {
            let error_message = format!(
                "cannot uninstall {} because it is the default version.",
                tool
            );
            let hint = "hint: run `golta default clear` or change default before uninstalling.";
            return Err(format!("{}\n{}", error_message, hint).into());
        }
    }

    // Remove the version
    fs::remove_dir_all(&version_dir)?;
    println!("Go {} has been uninstalled.", version);

    Ok(())
}
