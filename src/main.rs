mod cli;
mod core;
mod utils;

use clap::{Parser, Subcommand};

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
    Pin { tool: String },
    Default { tool: String },
    Run { tool: String, args: Vec<String> },
    Exec { tool: String, args: Vec<String> },
    List { tool: Option<String> },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Install { tool } => cli::install::run(tool),
        Commands::Pin { tool } => cli::pin::run(tool),
        Commands::Default { tool } => cli::default::run(tool),
        Commands::Run { tool, args } => cli::run::run(tool, args),
        Commands::Exec { tool, args } => cli::exec::run(tool, args),
        Commands::List { tool } => cli::list::run(tool),
    }
}
