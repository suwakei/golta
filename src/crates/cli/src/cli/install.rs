use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use semver::{Version, VersionReq};
use serde::Deserialize;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Cursor;
use std::path::Path;
use zip::ZipArchive;

#[cfg(target_os = "windows")]
const OS: &str = "windows-amd64";
#[cfg(target_os = "linux")]
const OS: &str = "linux-amd64";
#[cfg(target_os = "macos")]
const OS: &str = "darwin-amd64";

#[derive(Deserialize, Debug)]
struct GoVersionInfo {
    version: String,
    stable: bool,
}

pub async fn run(tool: String) {
    if let Err(e) = install_go(&tool).await {
        eprintln!("Error: {}", e);
    }
}

async fn install_go(tool: &str) -> Result<(), Box<dyn Error>> {
    let version_spec = if tool.starts_with("go@") {
        tool.trim_start_matches("go@")
    } else if tool == "go" {
        "latest"
    } else {
        return Err(
            "Invalid format. Use `golta install go` or `golta install go@<version>`.".into(),
        );
    };

    let version = resolve_go_version(version_spec).await?;

    println!("Installing Go version {}", version);

    // Use `home::home_dir` to get the home directory in a cross-platform way
    let home = home::home_dir().ok_or("Could not find home directory")?;
    let install_dir = home.join(".golta").join("versions").join(&version);

    if install_dir.exists() {
        println!("Go {} is already installed.", version);
        return Ok(());
    }

    fs::create_dir_all(&install_dir)?;

    // Download URL
    // Use .tar.gz for non-Windows OS
    #[cfg(not(target_os = "windows"))]
    let archive_format = "tar.gz";
    #[cfg(target_os = "windows")]
    let archive_format = "zip";

    let url = format!(
        "https://golang.org/dl/go{}.{}.{}",
        &version, OS, archive_format
    );
    println!("Downloading {} ...", url);

    let response = reqwest::get(&url).await?.error_for_status()?; // Check for 4xx, 5xx errors

    let total_size = response
        .content_length()
        .ok_or_else(|| format!("Failed to get content length from {}", &url))?;

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
        .progress_chars("#>-"));

    // Try to safely convert u64 to usize
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

    let bytes = downloaded_bytes;

    // Extract based on OS
    println!("Extracting...");
    #[cfg(target_os = "windows")]
    {
        extract_zip(&bytes, &install_dir)?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        use flate2::read::GzDecoder;
        use tar::Archive;
        let tar_gz_cursor = Cursor::new(&bytes);
        let tar = GzDecoder::new(tar_gz_cursor);
        let mut archive = Archive::new(tar);
        // Since it's extracted into `go/`, adjust the destination directory
        let temp_extract_dir = install_dir.join("go_temp");
        archive.unpack(&temp_extract_dir)?;
        fs::rename(temp_extract_dir.join("go"), install_dir.join("go"))?;
    }

    println!("Go {} installed to {:?}", version, install_dir);
    Ok(())
}

async fn resolve_go_version(spec: &str) -> Result<String, Box<dyn Error>> {
    println!("Finding matching Go version for \"{}\"...", spec);
    let versions: Vec<GoVersionInfo> = reqwest::get("https://go.dev/dl/?mode=json")
        .await?
        .json()
        .await?;

    if spec == "latest" {
        let latest_stable = versions
            .into_iter()
            .find(|v| v.stable)
            .ok_or("Could not find a stable Go version.")?;
        return Ok(latest_stable.version.trim_start_matches("go").to_string());
    }

    let available_versions: Vec<Version> = versions
        .iter()
        .filter_map(|v| Version::parse(v.version.trim_start_matches("go")).ok())
        .collect();

    // If a full version is specified, check for an exact match.
    if spec.matches('.').count() == 2 {
        let requested_version = Version::parse(spec)?;
        if available_versions.contains(&requested_version) {
            println!("Found exact match for version: {}", requested_version);
            return Ok(requested_version.to_string());
        } else {
            return Err(format!("Go version {} not found.", spec).into());
        }
    }

    // If a partial version is specified, find the latest matching patch version.
    let req = VersionReq::parse(&format!("~{}", spec))?;
    let matching_version = available_versions
        .into_iter()
        .filter(|v| req.matches(v))
        .max();

    match matching_version {
        Some(v) => {
            println!("Found matching version: {}", v);
            Ok(v.to_string())
        }
        None => Err(format!("No matching Go version found for spec '{}'", spec).into()),
    }
}

/// Extracts a byte slice as a zip archive
#[cfg(windows)]
fn extract_zip(bytes: &[u8], dest: &Path) -> std::io::Result<()> {
    let cursor = Cursor::new(bytes);
    let mut zip = ZipArchive::new(cursor)?;

    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;

        // Extract into the `go/` directory
        let outpath = dest.join("go").join(file.name());
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
    }

    Ok(())
}
