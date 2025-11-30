use std::error::Error;
use std::io::{self, Write};

pub fn run() {
    let mut stdout = io::stdout();
    if let Err(e) = setup_environment(&mut stdout) {
        eprintln!("Error: {}", e);
    }
}

fn setup_environment<W: Write>(writer: &mut W) -> Result<(), Box<dyn Error>> {
    writeln!(writer, "Configuring your shell for Golta...")?;

    let cargo_bin = home::home_dir()
        .ok_or("Could not find home directory")?
        .join(".cargo")
        .join("bin");

    writeln!(
        writer,
        "\nThis will add the Golta shims directory to your shell's PATH."
    )?;
    writeln!(
        writer,
        "This is required for Golta to intercept `go` commands."
    )?;

    #[cfg(windows)]
    {
        writeln!(
            writer,
            "\nPlease add the following directory to your user PATH environment variable:"
        )?;
        writeln!(writer, "  {}", cargo_bin.display())?;
        writeln!(
            writer,
            "\nYou will need to restart your terminal for the changes to take effect."
        )?;
    }

    #[cfg(not(windows))]
    {
        writeln!(
            writer,
            "\nPlease add the following line to your shell's startup file (e.g., ~/.bashrc, ~/.zshrc):"
        )?;
        writeln!(writer, "\n  export PATH=\"{}:$PATH\"", cargo_bin.display())?;
        writeln!(
            writer,
            "\nAfter adding the line, restart your terminal or run `source <your_shell_file>`."
        )?;
    }

    writeln!(writer, "\nSetup complete. Welcome to Golta!")?;

    Ok(())
}
