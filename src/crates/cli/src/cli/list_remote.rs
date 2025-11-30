use crate::shared::versions::{fetch_remote_versions, GoVersionInfo};
use std::error::Error;
use std::future::Future;
use std::io::Write;

pub async fn run() {
    let mut out = std::io::stdout();
    if let Err(e) = list_remote_go_versions(fetch_remote_versions, &mut out).await {
        eprintln!("Error: {}", e);
    }
}

async fn list_remote_go_versions<W, Fetch, Fut>(
    fetch_versions: Fetch,
    out: &mut W,
) -> Result<(), Box<dyn Error>>
where
    W: Write,
    Fetch: Fn() -> Fut,
    Fut: Future<Output = Result<Vec<GoVersionInfo>, Box<dyn Error>>>,
{
    writeln!(out, "Fetching available Go versions from go.dev...")?;

    let versions: Vec<GoVersionInfo> = fetch_versions().await?;
    render_versions(&versions, out)?;

    Ok(())
}

fn render_versions(versions: &[GoVersionInfo], out: &mut impl Write) -> Result<(), Box<dyn Error>> {
    writeln!(out, "\nAvailable Go versions:")?;

    for v in versions.iter() {
        let version_number = v.version.trim_start_matches("go");
        if v.stable {
            writeln!(out, "  {}", version_number)?;
        } else {
            writeln!(out, "  {} (unstable)", version_number)?;
        }
    }

    writeln!(
        out,
        "\nUse `golta install go@<version>` to install a specific version."
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_stable_and_unstable_versions() {
        let versions = vec![
            GoVersionInfo {
                version: "go1.22.1".into(),
                stable: true,
            },
            GoVersionInfo {
                version: "go1.23rc1".into(),
                stable: false,
            },
        ];
        let mut out = Vec::new();

        render_versions(&versions, &mut out).unwrap();

        let output = String::from_utf8(out).unwrap();
        assert!(output.contains("Available Go versions:"));
        assert!(output.contains("  1.22.1"));
        assert!(output.contains("  1.23rc1 (unstable)"));
        assert!(output.contains("Use `golta install go@<version>`"));
    }

    #[test]
    fn list_remote_accepts_injected_fetcher() {
        let versions = vec![GoVersionInfo {
            version: "go1.20.0".into(),
            stable: true,
        }];
        let mut out = Vec::new();

        let fake_fetcher = || async { Ok(versions.clone()) };
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            list_remote_go_versions(fake_fetcher, &mut out)
                .await
                .unwrap();
        });

        let output = String::from_utf8(out).unwrap();
        assert!(output.contains("Fetching available Go versions from go.dev..."));
        assert!(output.contains("  1.20.0"));
    }
}
