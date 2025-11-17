use std::env;
use std::error::Error;
use std::fs;
use std::process::Command;

pub fn run(tool: String, args: Vec<String>) {
    // エラーが発生した場合は、メッセージを表示して終了する
    if let Err(e) = exec_go(&tool, &args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn exec_go(tool: &str, args: &[String]) -> Result<(), Box<dyn Error>> {
    if tool != "go" {
        return Err("Only `go` is supported for exec currently.".into());
    }

    // 1. 実行すべきGoのバージョンを特定する (which と同じロジック)
    let version_str = {
        // 1a. プロジェクトでピン留めされたバージョンを探す
        let mut current_dir = env::current_dir()?;
        let mut pinned_version = None;

        loop {
            let pin_file_path = current_dir.join(".golta.json");
            if pin_file_path.exists() {
                let content = fs::read_to_string(pin_file_path)?;
                let json: serde_json::Value = serde_json::from_str(&content)?;
                if let Some(go_ver) = json.get("go").and_then(|v| v.as_str()) {
                    pinned_version = Some(go_ver.to_string());
                }
                break;
            }

            if !current_dir.pop() {
                break;
            }
        }

        // 1b. ピン留めされていなければ、グローバルなデフォルトバージョンを読む
        pinned_version.unwrap_or_else(|| {
            let default_path = home::home_dir().map(|home| {
                home.join(".golta")
                    .join("state")
                    .join("default.txt")
            });
            default_path
                .and_then(|path| fs::read_to_string(path).ok())
                .unwrap_or_default()
                .trim()
                .to_string()
        })
    };

    if version_str.is_empty() {
        return Err("No Go version is active. Use `golta pin` or `golta default`.".into());
    }

    let go_executable_name = if cfg!(windows) { "go.exe" } else { "go" };
    let home = home::home_dir().ok_or("Could not find home directory")?;
    let go_path = home
        .join(".golta")
        .join("versions")
        .join(version_str.trim_start_matches("go@"))
        .join("bin")
        .join(go_executable_name);

    let status = Command::new(go_path).args(args).status()?; // `?` 演算子でI/Oエラーをハンドリング

    // 子プロセスの終了コードをそのまま終了コードとして使う
    std::process::exit(status.code().unwrap_or(1));
}
