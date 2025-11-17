use std::env;
use std::error::Error;
use std::fs;

pub fn run(tool: String) {
    if let Err(e) = unpin_go(&tool) {
        eprintln!("Error: {}", e);
    }
}

fn unpin_go(tool: &str) -> Result<(), Box<dyn Error>> {
    if !tool.starts_with("go") {
        return Err("Only unpinning `go` is supported currently.".into());
    }

    let pin_file = env::current_dir()?.join(".golta.json");

    if pin_file.exists() {
        fs::remove_file(&pin_file)?;
        println!("Unpinned Go version in {}", pin_file.display());
    } else {
        println!("No Go version is pinned in this directory.");
    }

    Ok(())
}
