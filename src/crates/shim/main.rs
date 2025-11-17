use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process::{exit, Command};

fn main() {
    // main関数でエラーが発生した場合、その内容を標準エラー出力に表示して終了する
    if let Err(e) = run() {
        eprintln!("golta-shim error: {}", e);
        exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    // 1. 実行すべきGoのバージョンを特定する
    let version = {
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
                break; // .golta.json が見つかったら探索を終了
            }

            if !current_dir.pop() {
                break; // 親ディレクトリがなければ（ルートまで達したら）終了
            }
        }

        // 1b. ピン留めされていなければ、グローバルなデフォルトバージョンを読む
        pinned_version.unwrap_or_else(|| {
            let home = home::home_dir()
                .ok_or("Could not find home directory")
                .unwrap();
            let default_file = home.join(".golta").join("state").join("default.txt");
            fs::read_to_string(default_file)
                .unwrap_or_default()
                .trim()
                .to_string()
        })
    };

    // バージョンが特定できなかった場合はエラー
    if version.is_empty() {
        return Err("No Go version is set. Use `golta pin go@<version>` in your project, or `golta default go@<version>` globally.".into());
    }

    // 2. 実際のGoバイナリのパスを構築
    let home = home::home_dir().ok_or("Could not find home directory")?;
    let go_executable_name = if cfg!(windows) { "go.exe" } else { "go" };
    let real_go_path: PathBuf = home
        .join(".golta")
        .join("versions")
        .join(version.trim_start_matches("go@")) // "go@" プレフィックスを削除
        .join("bin")
        .join(go_executable_name);

    // 引数取得
    let args: Vec<String> = env::args_os()
        .skip(1)
        .map(|s| s.into_string().unwrap())
        .collect();

    // 3. 実際のGoバイナリを実行
    let status = Command::new(real_go_path).args(args).status()?;

    // 子プロセスの exit code をそのまま返す
    // シグナルで終了した場合(Unix系)も考慮し、デフォルトで1を返す
    exit(status.code().unwrap_or(1));
}
