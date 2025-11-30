use serde::{Deserialize, Serialize};
use std::error::Error;

/// Represents version information fetched from the Go download server.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct GoVersionInfo {
    pub version: String,
    pub stable: bool,
}

/// Fetches the list of available Go versions from the official Go website.
///
/// This function queries the JSON endpoint that includes all historical versions.
pub async fn fetch_remote_versions() -> Result<Vec<GoVersionInfo>, Box<dyn Error>> {
    fetch_remote_versions_from_url("https://go.dev/dl/?mode=json&include=all").await
}

/// Fetches versions from a specified endpoint. Allows injecting a test server URL.
pub async fn fetch_remote_versions_from_url(
    url: &str,
) -> Result<Vec<GoVersionInfo>, Box<dyn Error>> {
    let body = reqwest::get(url).await?.text().await?;
    parse_versions(&body)
}

/// Parses version information from JSON. Useful for offline tests.
pub fn parse_versions(json: &str) -> Result<Vec<GoVersionInfo>, Box<dyn Error>> {
    let versions = serde_json::from_str(json)?;
    Ok(versions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_versions_from_json() {
        let json = r#"
        [
            {"version": "go1.22.3", "stable": true},
            {"version": "go1.21.9", "stable": false}
        ]
        "#;

        let versions = parse_versions(json).unwrap();
        assert_eq!(
            versions,
            vec![
                GoVersionInfo {
                    version: "go1.22.3".into(),
                    stable: true
                },
                GoVersionInfo {
                    version: "go1.21.9".into(),
                    stable: false
                }
            ]
        );
    }

    #[test]
    fn fails_on_invalid_json() {
        let json = "not-json";
        assert!(parse_versions(json).is_err());
    }
}
