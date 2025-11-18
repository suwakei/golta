use std::env;
use std::error::Error;
use std::fs;

pub fn run() {
    if let Err(e) = unpin_go() {
        eprintln!("Error: {}", e);
    }
}

fn unpin_go() -> Result<(), Box<dyn Error>> {
    let pin_file = env::current_dir()?.join(".golta.json");

    if pin_file.exists() {
        fs::remove_file(&pin_file)?;
        println!("Removed pinned version for this project.");
    } else {
        println!("No Go version is pinned in this directory.");
    }

    Ok(())
}
