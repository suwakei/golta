use std::error::Error;
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

pub async fn run(tool: String) {
    if let Err(e) = install_go(&tool).await {
        eprintln!("Error: {}", e);
    }
}

async fn install_go(tool: &str) -> Result<(), Box<dyn Error>> {
    if !tool.starts_with("go@") {
        return Err(
            "Only Go installation is supported currently. Use format 'go@<version>'.".into(),
        );
    }

    let version = tool.trim_start_matches("go@");
    println!("Installing Go version {}", version);

    // `home::home_dir` を使ってクロスプラットフォームでホームディレクトリを取得
    let home = home::home_dir().ok_or("Could not find home directory")?;
    let install_dir = home.join(".golta").join("versions").join(version);

    if install_dir.exists() {
        println!("Go {} is already installed.", version);
        return Ok(());
    }

    std::fs::create_dir_all(&install_dir)?;

    // ダウンロード URL
    // Windows以外では .tar.gz を使うように修正
    #[cfg(not(target_os = "windows"))]
    let archive_format = "tar.gz";
    #[cfg(target_os = "windows")]
    let archive_format = "zip";

    let url = format!(
        "https://golang.org/dl/go{}.{}.{}",
        version, OS, archive_format
    );
    println!("Downloading {} ...", url);

    let response = reqwest::get(&url).await?.error_for_status()?;
    let bytes = response.bytes().await?;

    // OS ごとに展開
    #[cfg(target_os = "windows")]
    {
        extract_zip(&bytes, &install_dir)?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        use std::process::Command;
        // tar.gzを展開するライブラリを使うことで、tarコマンドへの依存をなくせる
        let temp_dir = tempfile::tempdir()?;
        let tar_path = temp_dir.path().join(format!("go{}.tar.gz", version));
        std::fs::write(&tar_path, &bytes)?;

        let status = Command::new("tar")
            .args([
                "-xzf",
                tar_path.to_str().ok_or("Invalid path")?,
                "-C",
                install_dir.to_str().ok_or("Invalid path")?,
                "--strip-components=1",
            ])
            .status()?;

        if !status.success() {
            return Err("Failed to extract Go tarball. Is 'tar' command installed?".into());
        }
    }

    println!("Go {} installed to {:?}", version, install_dir);
    Ok(())
}

/// バイト列を zip として展開
fn extract_zip(bytes: &[u8], dest: &Path) -> std::io::Result<()> {
    let cursor = Cursor::new(bytes);
    let mut zip = ZipArchive::new(cursor)?;

    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;

        // strip_prefixで先頭の "go/" ディレクトリを取り除く
        let outpath = match file.name().strip_prefix("go/") {
            Some(p) => dest.join(p),
            None => continue, // "go/" で始まらないエントリはスキップ
        };

        if file.is_dir() {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                std::fs::create_dir_all(p)?;
            }
            std::io::copy(&mut file, &mut File::create(&outpath)?)?;
        }
    }

    Ok(())
}
