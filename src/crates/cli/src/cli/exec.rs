use std::error::Error;
use std::fs;
use std::path::PathBuf;
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

    // `home::home_dir` を使ってクロスプラットフォームでホームディレクトリを安全に取得
    let home = home::home_dir().ok_or("Could not find home directory")?;
    let default_file = PathBuf::from(&home)
        .join(".golta")
        .join("state")
        .join("default.txt");
    let version = fs::read_to_string(default_file)
        .map_err(|_| "No default Go version is set. Use `golta default <version>` to set one.")?
        .trim()
        .to_string();

    if version.is_empty() {
        return Err("Default version file is empty.".into());
    }

    let go_executable_name = if cfg!(windows) { "go.exe" } else { "go" };
    let go_path = home
        .join(".golta")
        .join("versions")
        .join(version.trim_start_matches("go@")) // "go@" プレフィックスを削除
        .join("bin")
        .join(go_executable_name);

    let status = Command::new(go_path).args(args).status()?; // `?` 演算子でI/Oエラーをハンドリング

    // 子プロセスの終了コードをそのまま終了コードとして使う
    std::process::exit(status.code().unwrap_or(1));
}
