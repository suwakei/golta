mod cli;

use clap::{Parser, Subcommand};
use clap_complete::Shell;
use cli::{completions, default, exec, install, list, pin, run, setup, uninstall, unpin, which};

#[derive(Parser)]
#[command(name = "golta")]
#[command(version, about = "Golta CLI - A fast, simple Go version manager", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Download and install a tool version (e.g., go@1.23.0)", aliases = &["i", "fetch"])]
    Install {
        /// The tool and version to install (e.g., "go@1.23.0")
        tool: String,
    },
    #[command(about = "Uninstall a specific version of a tool", alias = "un")]
    Uninstall {
        /// The tool and version to uninstall (e.g., "go@1.23.0")
        tool: String,
    },
    #[command(about = "Set the global default version for a tool")]
    Default {
        /// The tool and version to set as default (e.g., "go@1.23.0")
        tool: String,
    },
    #[command(
        about = "Run a command with a one-time tool version, ignoring the current configuration"
    )]
    Run {
        /// The tool and version to run with (e.g., "go@1.23.0")
        tool: String,
        /// The arguments to pass to the command
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Execute a command using the currently active tool version")]
    Exec {
        /// The tool to execute (e.g., "go")
        tool: String,
        /// The arguments to pass to the command
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Pin a tool version to the current project (.golta.json)")]
    Pin {
        /// The tool and version to pin (e.g., "go@1.23.0")
        tool: String,
    },
    #[command(about = "Unpin the tool version from the current project")]
    Unpin,
    #[command(about = "Display the full path to the currently active tool executable")]
    Which {
        /// The tool to find (e.g., "go")
        tool: String,
    },
    #[command(about = "List all installed versions", alias = "ls")]
    List,
    #[command(about = "Generate shell completion scripts")]
    Completions {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
    #[command(about = "Configure your shell for Golta (run on first install)")]
    Setup,
    #[command(about = "Show the version of Golta")]
    Version,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Install { tool } => install::run(tool).await,
        Commands::Uninstall { tool } => uninstall::run(tool),
        Commands::Default { tool } => default::run(tool),
        Commands::Run { tool, args } => run::run(tool, args),
        Commands::Exec { tool, args } => exec::run(tool, args),
        Commands::Pin { tool } => pin::run(tool),
        Commands::Unpin => unpin::run(),
        Commands::Which { tool } => which::run(tool),
        Commands::List => list::run(),
        Commands::Completions { shell } => completions::run(shell),
        Commands::Setup => setup::run(),
        Commands::Version => {
            println!("golta {}", env!("CARGO_PKG_VERSION"));
        }
    }
}
