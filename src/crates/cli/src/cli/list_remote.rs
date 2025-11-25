use crate::shared::versions::{fetch_remote_versions, GoVersionInfo};
use std::error::Error;

pub async fn run() {
    if let Err(e) = list_remote_go_versions().await {
        eprintln!("Error: {}", e);
    }
}

async fn list_remote_go_versions() -> Result<(), Box<dyn Error>> {
    println!("Fetching available Go versions from go.dev...");

    let versions: Vec<GoVersionInfo> = fetch_remote_versions().await?;
    println!("\nAvailable Go versions:");

    // Show all versions, and mark unstable ones.
    for v in versions.iter() {
        let version_number = v.version.trim_start_matches("go");
        if v.stable {
            println!("  {}", version_number);
        } else {
            println!("  {} (unstable)", version_number);
        }
    }

    println!("\nUse `golta install go@<version>` to install a specific version.");

    Ok(())
}
