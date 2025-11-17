use std::fs;
use std::path::PathBuf;

pub fn run(tool: String) {
    if !tool.starts_with("go@") {
        println!("Only Go default version is supported currently.");
        return;
    }

    let version = tool.trim_start_matches("go@");

    // ~/.golta/state/default.txt
    let home = std::env::var("HOME").unwrap();
    let state_dir = PathBuf::from(&home).join(".golta").join("state");
    fs::create_dir_all(&state_dir).unwrap();

    let default_file = state_dir.join("default.txt");
    fs::write(default_file, version).unwrap();

    println!("Set Go default version to {}", version);
}
