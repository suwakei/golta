mod cli;

use clap::{Parser, Subcommand};
use cli::{install, default};

#[derive(Parser)]
#[command(name = "golta")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Install { tool: String },
    Default { tool: String },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Install { tool } => install::run(tool).await,
        Commands::Default { tool } => default::run(tool),
    }
}
