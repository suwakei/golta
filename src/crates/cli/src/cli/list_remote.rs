use crate::cli::install::get_tool_info;
use crate::shared::versions::{fetch_remote_versions, GoVersionInfo};
use std::error::Error;
use std::fs;
use std::future::Future;
use std::io::Write;
use std::path::PathBuf;

pub async fn run(tool_opt: Option<String>) {
    let tool = tool_opt.unwrap_or_else(|| "go".to_string());

    let mut out = std::io::stdout();
    let home = match home::home_dir() {
        Some(path) => path,
        None => {
            eprintln!("Error: Could not find home directory");
            return;
        }
    };

    let cache = FsRemoteVersionsCache::new(home, &tool);

    if tool == "go" {
        if let Err(e) = list_remote_versions("Go", fetch_remote_versions, &cache, &mut out).await {
            eprintln!("Error: {}", e);
        }
    } else {
        match get_tool_info(&tool) {
            Some((_, module_path)) => {
                let module = module_path.to_string();
                let fetcher = || fetch_tool_versions(module.clone());
                if let Err(e) = list_remote_versions(&tool, fetcher, &cache, &mut out).await {
                    eprintln!("Error: {}", e);
                }
            }
            None => {
                eprintln!("Error: Unknown tool '{}'. Supported tools: gopls, dlv, air, staticcheck, golangci-lint", tool);
            }
        }
    }
}

async fn list_remote_versions<W, Fetch, Fut>(
    tool_name: &str,
    fetch_versions: Fetch,
    cache: &impl RemoteVersionsCache,
    out: &mut W,
) -> Result<(), Box<dyn Error>>
where
    W: Write,
    Fetch: Fn() -> Fut,
    Fut: Future<Output = Result<Vec<GoVersionInfo>, Box<dyn Error>>>,
{
    let cached_versions = cache.read_cache().unwrap_or(None);
    let fetched = fetch_versions().await;

    match fetched {
        Ok(remote_versions) => {
            let use_cache = cached_versions
                .as_ref()
                .and_then(|cached| cached.first())
                .zip(remote_versions.first())
                .map(|(cached_latest, remote_latest)| cached_latest == remote_latest)
                .unwrap_or(false);

            if use_cache {
                writeln!(
                    out,
                    "Latest {} versions unchanged; showing cached results.",
                    tool_name
                )?;
                render_versions(cached_versions.as_ref().unwrap(), out)?;
                return Ok(());
            }

            writeln!(out, "Fetching available {} versions...", tool_name)?;
            render_versions(&remote_versions, out)?;
            if let Err(e) = cache.write_cache(&remote_versions) {
                writeln!(out, "Warning: failed to update cache ({})", e).ok();
            }
            Ok(())
        }
        Err(fetch_error) => {
            if let Some(cached) = cached_versions {
                writeln!(
                    out,
                    "Failed to fetch latest versions ({}). Showing cached results.",
                    fetch_error
                )?;
                render_versions(&cached, out)
            } else {
                Err(fetch_error)
            }
        }
    }
}

async fn fetch_tool_versions(package: String) -> Result<Vec<GoVersionInfo>, Box<dyn Error>> {
    let url = format!("https://proxy.golang.org/{}/@v/list", package);
    let response = reqwest::get(&url).await?.error_for_status()?.text().await?;

    let mut versions: Vec<GoVersionInfo> = response
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let v = line.trim();
            let stable = !v.contains("rc") && !v.contains("beta") && !v.contains("alpha");
            GoVersionInfo {
                version: v.to_string(),
                stable,
            }
        })
        .collect();

    // Proxy list is typically oldest to newest. Reverse to show newest first.
    versions.reverse();
    Ok(versions)
}

fn render_versions(versions: &[GoVersionInfo], out: &mut impl Write) -> Result<(), Box<dyn Error>> {
    writeln!(out, "\nAvailable versions:")?;

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
        "\nUse `golta install <tool>@<version>` to install a specific version."
    )?;

    Ok(())
}

trait RemoteVersionsCache {
    fn read_cache(&self) -> Result<Option<Vec<GoVersionInfo>>, Box<dyn Error>>;
    fn write_cache(&self, versions: &[GoVersionInfo]) -> Result<(), Box<dyn Error>>;
}

struct FsRemoteVersionsCache {
    path: PathBuf,
}

impl FsRemoteVersionsCache {
    fn new(home: PathBuf, tool: &str) -> Self {
        let filename = if tool == "go" {
            "remote_versions.json".to_string()
        } else {
            format!("remote_versions_{}.json", tool)
        };
        let path = home.join(".golta").join("cache").join(filename);
        Self { path }
    }
}

impl RemoteVersionsCache for FsRemoteVersionsCache {
    fn read_cache(&self) -> Result<Option<Vec<GoVersionInfo>>, Box<dyn Error>> {
        if !self.path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&self.path)?;
        let versions: Vec<GoVersionInfo> = serde_json::from_str(&content)?;
        Ok(Some(versions))
    }

    fn write_cache(&self, versions: &[GoVersionInfo]) -> Result<(), Box<dyn Error>> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(versions)?;
        fs::write(&self.path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

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
        assert!(output.contains("Available versions:"));
        assert!(output.contains("  1.22.1"));
        assert!(output.contains("  1.23rc1 (unstable)"));
        assert!(output.contains("Use `golta install <tool>@<version>`"));
    }

    #[test]
    fn list_remote_accepts_injected_fetcher() {
        let versions = vec![GoVersionInfo {
            version: "go1.20.0".into(),
            stable: true,
        }];
        let mut out = Vec::new();
        let cache = MockCache::default();

        let fake_fetcher = || async { Ok(versions.clone()) };
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            list_remote_versions("Go", fake_fetcher, &cache, &mut out)
                .await
                .unwrap();
        });

        let output = String::from_utf8(out).unwrap();
        assert!(output.contains("Fetching available Go versions..."));
        assert!(output.contains("  1.20.0"));
    }

    #[test]
    fn uses_cache_when_remote_latest_matches() {
        let cached = vec![GoVersionInfo {
            version: "go1.20.0".into(),
            stable: true,
        }];
        let cache = MockCache::with_data(cached.clone());
        let fetcher = || async {
            Ok(vec![GoVersionInfo {
                version: "go1.20.0".into(),
                stable: true,
            }])
        };
        let mut out = Vec::new();

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            list_remote_versions("Go", fetcher, &cache, &mut out)
                .await
                .unwrap();
        });

        let output = String::from_utf8(out).unwrap();
        assert!(output.contains("cached results"));
        assert!(!output.contains("Fetching available Go versions..."));
        assert!(output.contains("  1.20.0"));
        assert_eq!(cache.write_calls(), 0, "should not rewrite cache");
    }

    #[test]
    fn falls_back_to_cache_on_fetch_error() {
        let cached = vec![GoVersionInfo {
            version: "go1.19.0".into(),
            stable: true,
        }];
        let cache = MockCache::with_data(cached.clone());
        let failing_fetcher = || async { Err::<Vec<GoVersionInfo>, _>("network error".into()) };
        let mut out = Vec::new();

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            list_remote_versions("Go", failing_fetcher, &cache, &mut out)
                .await
                .unwrap();
        });

        let output = String::from_utf8(out).unwrap();
        assert!(output.contains("Showing cached results."));
        assert!(output.contains("  1.19.0"));
    }

    #[test]
    fn writes_cache_when_remote_has_new_version() {
        let cached = vec![GoVersionInfo {
            version: "go1.20.0".into(),
            stable: true,
        }];
        let cache = MockCache::with_data(cached);
        let remote = vec![GoVersionInfo {
            version: "go1.21.0".into(),
            stable: true,
        }];
        let fetcher = || async { Ok(remote.clone()) };
        let mut out = Vec::new();

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            list_remote_versions("Go", fetcher, &cache, &mut out)
                .await
                .unwrap();
        });

        assert_eq!(cache.write_calls(), 1, "cache should be updated");
        let output = String::from_utf8(out).unwrap();
        assert!(output.contains("  1.21.0"));
    }

    #[derive(Default)]
    struct MockCache {
        stored: RefCell<Option<Vec<GoVersionInfo>>>,
        writes: RefCell<usize>,
    }

    impl MockCache {
        fn with_data(data: Vec<GoVersionInfo>) -> Self {
            Self {
                stored: RefCell::new(Some(data)),
                writes: RefCell::new(0),
            }
        }

        fn write_calls(&self) -> usize {
            *self.writes.borrow()
        }
    }

    impl RemoteVersionsCache for MockCache {
        fn read_cache(&self) -> Result<Option<Vec<GoVersionInfo>>, Box<dyn Error>> {
            Ok(self.stored.borrow().clone())
        }

        fn write_cache(&self, versions: &[GoVersionInfo]) -> Result<(), Box<dyn Error>> {
            *self.writes.borrow_mut() += 1;
            *self.stored.borrow_mut() = Some(versions.to_vec());
            Ok(())
        }
    }
}
