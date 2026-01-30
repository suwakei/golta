use crate::shared::os_info::get_os_arch_and_format;
use crate::shared::versions::{fetch_remote_versions, GoVersionInfo};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
use std::error::Error;
use std::fs;
#[cfg(windows)]
use std::fs::File;
use std::future::Future;
use std::io::{self, Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
#[cfg(windows)]
use zip::ZipArchive;

pub async fn run(tool_arg: String) {
    let home = match home::home_dir() {
        Some(path) => path,
        None => {
            eprintln!("Error: Could not find home directory");
            std::process::exit(1);
        }
    };

    let mut stdout = io::stdout();

    let (tool, version_spec) = match parse_tool_and_version(&tool_arg) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    if tool != "go" {
        if let Err(e) = install_go_tool(&tool, &version_spec, &home, &mut stdout).await {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
        return;
    }

    if let Err(e) = install_go(
        &version_spec,
        &home,
        fetch_remote_versions,
        download_with_progress,
        &mut stdout,
    )
    .await
    {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn install_go<W, FetchVersions, FetchVersionsFut, DownloadBytes, DownloadBytesFut>(
    version_spec: &str,
    home: &Path,
    fetch_versions: FetchVersions,
    download_bytes: DownloadBytes,
    writer: &mut W,
) -> Result<(), Box<dyn Error>>
where
    W: Write,
    FetchVersions: Fn() -> FetchVersionsFut,
    FetchVersionsFut: Future<Output = Result<Vec<GoVersionInfo>, Box<dyn Error>>>,
    DownloadBytes: Fn(String) -> DownloadBytesFut,
    DownloadBytesFut: Future<Output = Result<Vec<u8>, Box<dyn Error>>>,
{
    let version = resolve_go_version(version_spec, fetch_versions, writer).await?;

    writeln!(writer, "Installing Go version {}", version)?;

    let install_dir = build_install_dir(home, &version);

    if install_dir.exists() {
        writeln!(writer, "Go {} is already installed.", version)?;
        return Ok(());
    }

    let (os_arch, archive_format) = get_os_arch_and_format();
    let url = build_download_url(&version, os_arch, archive_format);
    writeln!(writer, "Downloading {} ...", url)?;

    let bytes = download_bytes(url).await?;

    fs::create_dir_all(&install_dir)?;

    writeln!(writer, "Extracting...")?;
    let extract_pb = ProgressBar::new_spinner();
    extract_pb.set_style(ProgressStyle::with_template(
        "{spinner:.green} extracting {msg}",
    )?);
    #[cfg(target_os = "windows")]
    {
        extract_zip(&bytes, &install_dir, &extract_pb)?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        extract_tar_gz(&bytes, &install_dir, &extract_pb)?;
    }
    extract_pb.finish_with_message("Extracted");

    writeln!(writer, "Go {} installed to {:?}", version, install_dir)?;
    Ok(())
}

async fn install_go_tool(
    tool: &str,
    version: &str,
    home: &Path,
    writer: &mut impl Write,
) -> Result<(), Box<dyn Error>> {
    let (package_path, module_path) = get_tool_info(tool).ok_or(format!(
        "Unknown tool: '{}'. Currently supported tools: gopls, dlv, air, staticcheck, golangci-lint",
        tool
    ))?;

    let resolved_version = if version == "latest" {
        resolve_latest_tool_version(module_path, writer).await?
    } else {
        version.to_string()
    };

    let install_dir = home
        .join(".golta")
        .join("versions")
        .join(tool)
        .join(&resolved_version);
    let bin_dir = install_dir.join("bin");

    if bin_dir.exists() {
        writeln!(
            writer,
            "{} {} is already installed.",
            tool, resolved_version
        )?;
        return Ok(());
    }

    writeln!(writer, "Installing {}@{}...", tool, resolved_version)?;

    let status = Command::new("go")
        .arg("install")
        .arg(format!("{}@{}", package_path, resolved_version))
        .env("GOBIN", &bin_dir)
        .status()
        .map_err(|e| format!("Failed to execute 'go install': {}. Make sure 'go' is in your PATH or installed via golta.", e))?;

    if !status.success() {
        return Err(format!("Failed to install {}@{}", tool, resolved_version).into());
    }

    writeln!(
        writer,
        "Installed {} {} to {:?}",
        tool, resolved_version, bin_dir
    )?;

    Ok(())
}

fn parse_tool_and_version(input: &str) -> Result<(String, String), Box<dyn Error>> {
    if input == "go" {
        if let Some(v) = read_go_mod_version() {
            return Ok(("go".to_string(), v));
        } else {
            return Ok(("go".to_string(), "latest".to_string()));
        }
    }
    if input == "go@mod" {
        let v = read_go_mod_version().ok_or("Could not find 'go <version>' in go.mod")?;
        return Ok(("go".to_string(), v));
    }

    if let Some((tool, version)) = input.split_once('@') {
        return Ok((tool.to_string(), version.to_string()));
    }

    Ok((input.to_string(), "latest".to_string()))
}

pub(crate) fn get_tool_info(tool: &str) -> Option<(&str, &str)> {
    match tool {
        "gopls" => Some(("golang.org/x/tools/gopls", "golang.org/x/tools/gopls")),
        "dlv" => Some((
            "github.com/go-delve/delve/cmd/dlv",
            "github.com/go-delve/delve",
        )),
        "air" => Some(("github.com/air-verse/air", "github.com/air-verse/air")),
        "staticcheck" => Some(("honnef.co/go/tools/cmd/staticcheck", "honnef.co/go/tools")),
        "golangci-lint" => Some((
            "github.com/golangci/golangci-lint/cmd/golangci-lint",
            "github.com/golangci/golangci-lint",
        )),
        _ => None,
    }
}

#[derive(Deserialize)]
struct ProxyVersion {
    #[serde(rename = "Version")]
    version: String,
}

async fn resolve_latest_tool_version(
    package: &str,
    writer: &mut impl Write,
) -> Result<String, Box<dyn Error>> {
    writeln!(writer, "Resolving latest version for {}...", package)?;
    let url = format!("https://proxy.golang.org/{}/@latest", package);
    let response = reqwest::get(&url).await?.error_for_status()?;
    let proxy_version: ProxyVersion = response.json().await?;
    Ok(proxy_version.version)
}

async fn resolve_go_version<F, Fut>(
    spec: &str,
    fetch_versions: F,
    writer: &mut impl Write,
) -> Result<String, Box<dyn Error>>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<Vec<GoVersionInfo>, Box<dyn Error>>>,
{
    writeln!(writer, "Finding matching Go version for \"{}\"...", spec)?;
    let remote_versions: Vec<GoVersionInfo> = fetch_versions().await?;

    resolve_go_version_from_list(spec, &remote_versions, writer)
}

fn resolve_go_version_from_list(
    spec: &str,
    versions: &[GoVersionInfo],
    writer: &mut impl Write,
) -> Result<String, Box<dyn Error>> {
    if spec == "latest" {
        let latest_stable = versions
            .iter()
            .find(|v| v.stable)
            .ok_or("Could not find a stable Go version.")?;
        return Ok(latest_stable.version.trim_start_matches("go").to_string());
    }

    let found_version = versions
        .iter()
        .find(|v| v.version.trim_start_matches("go") == spec);

    match found_version {
        Some(info) => {
            let version_str = info.version.trim_start_matches("go");
            writeln!(writer, "Found matching version: {}", version_str).ok();
            Ok(version_str.to_string())
        }
        None => Err(format!(
            "Go version '{}' not found. Please specify an exact version from `golta list-remote`.",
            spec
        )
        .into()),
    }
}

fn build_install_dir(home: &Path, version: &str) -> PathBuf {
    home.join(".golta").join("versions").join(version)
}

fn build_download_url(version: &str, os_arch: &str, archive_format: &str) -> String {
    format!(
        "https://golang.org/dl/go{}.{}.{}",
        version, os_arch, archive_format
    )
}

async fn download_with_progress(url: String) -> Result<Vec<u8>, Box<dyn Error>> {
    let response = reqwest::get(&url).await?.error_for_status()?;

    let total_size = response
        .content_length()
        .ok_or_else(|| format!("Failed to get content length from {}", &url))?;

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
        .progress_chars("#>-"));

    let capacity = total_size
        .try_into()
        .map_err(|_| "File size is too large to fit in memory on this system.".to_string())?;
    let mut downloaded_bytes = Vec::with_capacity(capacity);
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        downloaded_bytes.extend_from_slice(&chunk);
        pb.inc(chunk.len() as u64);
    }

    pb.finish_with_message("Downloaded");

    Ok(downloaded_bytes)
}

#[cfg(not(target_os = "windows"))]
fn extract_tar_gz(
    bytes: &[u8],
    install_dir: &Path,
    pb: &ProgressBar,
) -> Result<(), Box<dyn Error>> {
    use flate2::read::GzDecoder;
    use tar::Archive;
    let tar_gz_cursor = Cursor::new(bytes);
    let tar = GzDecoder::new(tar_gz_cursor);
    let mut archive = Archive::new(tar);
    let temp_extract_dir = install_dir.join("go_temp");

    if !temp_extract_dir.exists() {
        fs::create_dir_all(&temp_extract_dir)?;
    }

    archive.set_preserve_permissions(false);
    archive.set_preserve_mtime(false);

    for entry in archive.entries()? {
        let mut entry = entry?;
        entry.unpack_in(&temp_extract_dir)?;
        pb.set_message("...");
        pb.tick();
    }

    let source_path = temp_extract_dir.join("go");
    let destination_path = install_dir.join("go");

    if !source_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Source directory not found",
        )
        .into());
    }
    if destination_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "Destination directory already exists",
        )
        .into());
    }

    fs::rename(temp_extract_dir.join("go"), install_dir.join("go"))?;
    Ok(())
}

/// Extracts a byte slice as a zip archive
#[cfg(windows)]
fn extract_zip(bytes: &[u8], dest: &Path, pb: &ProgressBar) -> std::io::Result<()> {
    let cursor = Cursor::new(bytes);
    let mut zip = ZipArchive::new(cursor)?;
    let entries = zip.len();
    pb.set_length(entries as u64);
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} [{pos}/{len}] extracting {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );

    for i in 0..entries {
        let mut file = zip.by_index(i)?;
        let name = file.name().to_string();

        // Extract into the `go/` directory
        let outpath = dest.join(file.name());
        if file.name().is_empty() {
            continue;
        };

        if file.is_dir() {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                fs::create_dir_all(p)?;
            }
            std::io::copy(&mut file, &mut File::create(outpath)?)?;
        }
        pb.set_message(name);
        pb.inc(1);
    }

    Ok(())
}

fn read_go_mod_version() -> Option<String> {
    let path = std::env::current_dir().ok()?.join("go.mod");
    let content = fs::read_to_string(path).ok()?;
    content
        .lines()
        .find(|line| line.starts_with("go "))
        .map(|line| line.trim_start_matches("go ").trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tool_and_version_handles_latest() {
        assert_eq!(
            parse_tool_and_version("go").unwrap(),
            ("go".to_string(), "latest".to_string())
        ); // Assuming no go.mod
        assert_eq!(
            parse_tool_and_version("go@latest").unwrap(),
            ("go".to_string(), "latest".to_string())
        );
        assert_eq!(
            parse_tool_and_version("go@1.22.3").unwrap(),
            ("go".to_string(), "1.22.3".to_string())
        );
        assert_eq!(
            parse_tool_and_version("air").unwrap(),
            ("air".to_string(), "latest".to_string())
        );
        assert_eq!(
            parse_tool_and_version("air@v1.0").unwrap(),
            ("air".to_string(), "v1.0".to_string())
        );
    }

    #[test]
    fn builds_download_url_with_os_and_format() {
        let url = build_download_url("1.22.3", "linux-amd64", "tar.gz");
        assert_eq!(url, "https://golang.org/dl/go1.22.3.linux-amd64.tar.gz");
    }

    #[test]
    fn resolves_latest_version() {
        let versions = vec![
            GoVersionInfo {
                version: "go1.21.9".into(),
                stable: false,
            },
            GoVersionInfo {
                version: "go1.22.3".into(),
                stable: true,
            },
        ];

        let mut buffer = Vec::new();
        let resolved = resolve_go_version_from_list("latest", &versions, &mut buffer).unwrap();
        assert_eq!(resolved, "1.22.3");
    }

    #[test]
    fn resolves_exact_version() {
        let versions = vec![GoVersionInfo {
            version: "go1.20.1".into(),
            stable: true,
        }];

        let mut buffer = Vec::new();
        let resolved = resolve_go_version_from_list("1.20.1", &versions, &mut buffer).unwrap();
        assert_eq!(resolved, "1.20.1");
    }

    #[test]
    fn errors_when_version_not_found() {
        let versions = vec![GoVersionInfo {
            version: "go1.20.1".into(),
            stable: true,
        }];

        let mut buffer = Vec::new();
        let err = resolve_go_version_from_list("1.99.0", &versions, &mut buffer).unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn skips_install_if_already_exists() {
        let home = temp_home();
        let version = "1.21.0";
        let install_dir = home.join(".golta").join("versions").join(version);
        fs::create_dir_all(&install_dir).unwrap();

        let mut buffer = Vec::new();

        let fetcher = || async {
            Ok(vec![GoVersionInfo {
                version: format!("go{}", version),
                stable: true,
            }])
        };

        let downloader = |_| async {
            panic!("Downloader should not be called");
            #[allow(unreachable_code)]
            Ok(vec![])
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            install_go(version, &home, fetcher, downloader, &mut buffer)
                .await
                .unwrap();
        });

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains(&format!("Go {} is already installed.", version)));

        fs::remove_dir_all(home).unwrap();
    }

    fn temp_home() -> PathBuf {
        let mut path = std::env::temp_dir();
        let unique = format!(
            "golta_install_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        path.push(unique);
        path
    }
}
