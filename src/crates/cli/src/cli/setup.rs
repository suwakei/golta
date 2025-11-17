use std::error::Error;

pub fn run() {
    if let Err(e) = setup_environment() {
        eprintln!("Error: {}", e);
    }
}

fn setup_environment() -> Result<(), Box<dyn Error>> {
    println!("Configuring your shell for Golta...");

    let cargo_bin = home::home_dir()
        .ok_or("Could not find home directory")?
        .join(".cargo")
        .join("bin");

    println!("\nThis will add the Golta shims directory to your shell's PATH.");
    println!("This is required for Golta to intercept `go` commands.");

    #[cfg(windows)]
    {
        println!("\nPlease add the following directory to your user PATH environment variable:");
        println!("  {}", cargo_bin.display());
        println!("\nYou will need to restart your terminal for the changes to take effect.");
    }

    #[cfg(not(windows))]
    {
        println!("\nPlease add the following line to your shell's startup file (e.g., ~/.bashrc, ~/.zshrc):");
        println!("\n  export PATH=\"{}:$PATH\"", cargo_bin.display());
        println!(
            "\nAfter adding the line, restart your terminal or run `source <your_shell_file>`."
        );
    }

    println!("\nSetup complete. Welcome to Golta!");

    Ok(())
}
