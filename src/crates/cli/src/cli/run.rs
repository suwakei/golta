use std::error::Error;
use std::process::Command;

pub fn run(tool: String, args: Vec<String>) {
    // エラーが発生した場合は、メッセージを表示して終了する
    if let Err(e) = run_go(&tool, &args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run_go(tool: &str, args: &[String]) -> Result<(), Box<dyn Error>> {
    if !tool.starts_with("go@") {
        return Err("Only Go run is supported currently. Use format 'go@<version>'.".into());
    }

    let version = tool.trim_start_matches("go@");

    // `home::home_dir` を使ってクロスプラットフォームでホームディレクトリを安全に取得
    let home = home::home_dir().ok_or("Could not find home directory")?;

    let go_executable_name = if cfg!(windows) { "go.exe" } else { "go" };
    let go_path = home
        .join(".golta")
        .join("versions")
        .join(version)
        .join("bin")
        .join(go_executable_name);

    let status = Command::new(go_path).args(args).status()?; // `?` 演算子でI/Oエラーをハンドリング

    // 子プロセスの終了コードをそのまま終了コードとして使う
    std::process::exit(status.code().unwrap_or(1));
}
