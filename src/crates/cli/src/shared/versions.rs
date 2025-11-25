use serde::Deserialize;
use std::error::Error;

/// Represents version information fetched from the Go download server.
#[derive(Deserialize, Debug, Clone)]
pub struct GoVersionInfo {
    pub version: String,
    pub stable: bool,
}

/// Fetches the list of available Go versions from the official Go website.
///
/// This function queries the JSON endpoint that includes all historical versions.
pub async fn fetch_remote_versions() -> Result<Vec<GoVersionInfo>, Box<dyn Error>> {
    let versions: Vec<GoVersionInfo> = reqwest::get("https://go.dev/dl/?mode=json&include=all")
        .await?
        .json()
        .await?;
    Ok(versions)
}
