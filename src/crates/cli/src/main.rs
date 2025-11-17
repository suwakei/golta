mod cli;

use clap::{Parser, Subcommand};
use cli::{default, exec, install, list, pin, run, uninstall};

#[derive(Parser)]
#[command(name = "golta")]
#[command(about = "Golta CLI - Go Version Manager inspired by Volta", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Install { tool: String },
    Uninstall { tool: String },
    Default { tool: String },
    Run { tool: String, args: Vec<String> },
    Exec { tool: String, args: Vec<String> },
    Pin { tool: String },
    List,
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
        Commands::List => list::run(),
    }
}
