use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::fs;

#[derive(Serialize, Deserialize)]
struct PinFile {
    go: String,
}

pub fn run(tool: String) {
    if let Err(e) = pin_go_version(&tool) {
        eprintln!("Error: {}", e);
    }
}

fn pin_go_version(tool: &str) -> Result<(), Box<dyn Error>> {
    if !tool.starts_with("go@") {
        return Err("Invalid format. Use `golta pin go@<version>`.".into());
    }

    let version = tool.trim_start_matches("go@");

    // プロジェクトルート判定（カレントディレクトリ）
    let project_dir = env::current_dir()?;
    let pin_file = project_dir.join(".golta.json");

    let pin = PinFile {
        go: version.to_string(),
    };
    let json = serde_json::to_string_pretty(&pin)?;
    fs::write(&pin_file, json)?;

    println!("Pinned Go version {} to {}", version, pin_file.display());
    Ok(())
}
