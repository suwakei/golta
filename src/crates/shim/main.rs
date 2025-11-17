use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::process::{Command, exit};

fn main() {
    // main関数でエラーが発生した場合、その内容を標準エラー出力に表示して終了する
    if let Err(e) = run() {
        eprintln!("golta-shim error: {}", e);
        exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    // 1. HOME を取得
    // homeクレートを使うことで、Windows, macOS, Linuxでより確実にホームディレクトリを取得できる
    let home = home::home_dir().ok_or("Could not find home directory")?;

    // ~/.golta/state/default.txt
    let version_file = home
        .join(".golta")
        .join("state")
        .join("default.txt");

    // 現在の Go バージョンを読む
    // .expect() の代わりに ? 演算子を使い、エラーメッセージをカスタマイズする
    let version = std::fs::read_to_string(&version_file)
        .map_err(|_| "No default Go version is set. Use `golta default <version>` to set one.")?
        .trim().to_string();

    // バージョンが空文字列でないことを確認
    if version.is_empty() {
        return Err("Default version file is empty.".into());
    }

    // 実際の Go バイナリパス
    // OSによって実行ファイル名が異なる場合に対応 (例: Windowsでは .exe)
    let go_executable_name = if cfg!(windows) { "go.exe" } else { "go" };
    let real_go_path = home
        .join(".golta")
        .join("versions")
        .join(&version)
        .join("bin")
        .join(go_executable_name);

    // 引数取得
    let args: Vec<String> = env::args().skip(1).collect();

    // 実行
    let status = Command::new(real_go_path)
        .args(args)
        .status()?; // ここでも ? を使ってI/Oエラーをハンドリング

    // 子プロセスの exit code をそのまま返す
    // シグナルで終了した場合(Unix系)も考慮し、デフォルトで1を返す
    exit(status.code().unwrap_or(1));
}
