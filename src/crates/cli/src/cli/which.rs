use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

pub fn run(tool: String) {
    if let Err(e) = which_go(&tool) {
        eprintln!("Error: {}", e);
    }
}

fn which_go(tool: &str) -> Result<(), Box<dyn Error>> {
    if tool != "go" {
        return Err("Only `go` is supported for which currently.".into());
    }

    // 1. 実行すべきGoのバージョンを特定する (shim と同じロジック)
    let version = {
        // 1a. プロジェクトでピン留めされたバージョンを探す
        let mut current_dir = env::current_dir()?;
        let mut pinned_version = None;

        loop {
            let pin_file_path = current_dir.join(".golta.json");
            if pin_file_path.exists() {
                let content = fs::read_to_string(pin_file_path)?;
                // serde_json::from_str は Result を返すので ? でエラー処理
                let json: serde_json::Value = serde_json::from_str(&content)?;
                if let Some(go_ver) = json.get("go").and_then(|v| v.as_str()) {
                    pinned_version = Some(go_ver.to_string());
                }
                break; // .golta.json が見つかったら探索を終了
            }

            if !current_dir.pop() {
                break; // 親ディレクトリがなければ（ルートまで達したら）終了
            }
        }

        // 1b. ピン留めされていなければ、グローバルなデフォルトバージョンを読む
        pinned_version.unwrap_or_else(|| {
            // ここでの .unwrap() はパニックの可能性があるため、Result を返すようにする
            home::home_dir()
                .ok_or_else(|| "Could not find home directory".to_string())
                .and_then(|home| {
                    let default_file = home.join(".golta").join("state").join("default.txt");
                    fs::read_to_string(default_file).map_err(|e| e.to_string())
                })
                .unwrap_or_default() // エラーの場合は空文字列を返す
                .trim()
                .to_string()
        })
    };

    if version.is_empty() {
        return Err("No Go version is active. Use `golta pin` or `golta default`.".into());
    }

    // 2. 実際のGoバイナリのパスを構築
    let home = home::home_dir().ok_or("Could not find home directory")?;
    let go_executable_name = if cfg!(windows) { "go.exe" } else { "go" };
    let go_path: PathBuf = home
        .join(".golta")
        .join("versions")
        .join(version.trim_start_matches("go@"))
        .join("bin")
        .join(go_executable_name);

    println!("{}", go_path.display());
    Ok(())
}
