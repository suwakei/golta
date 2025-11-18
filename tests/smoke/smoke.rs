use std::env;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

// テスト対象のGoのバージョン
const GO_VERSION: &str = "1.22.0";
const GO_VERSION_ALT: &str = "1.21.0";

/// スモークテストのセットアップとクリーンアップを管理する構造体
struct SmokeTest {
    /// `golta`が使用する一時的なホームディレクトリ
    home_dir: TempDir,
    /// テスト中にインストールしたバージョンを記録
    installed_versions: Vec<String>,
    /// `golta`実行ファイルのパス
    golta_bin: String,
}

impl SmokeTest {
    /// テスト環境をセットアップする
    fn setup() -> Self {
        // `cargo test`によってビルドされた`golta`バイナリのパスを取得
        let golta_bin = env!("CARGO_BIN_EXE_golta").to_string();
        // テスト専用の一時的なホームディレクトリを作成
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let mut test = SmokeTest {
            home_dir: temp_dir,
            installed_versions: Vec::new(),
            golta_bin,
        };

        // テストで使うGoのバージョンをインストール
        test.install_version(GO_VERSION);
        test.install_version(GO_VERSION_ALT);

        test
    }

    /// `golta`コマンドを実行するヘルパー
    fn golta(&self, args: &[&str]) -> Output {
        Command::new(&self.golta_bin)
            .args(args)
            .env("HOME", self.home_dir.path())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Failed to execute command")
    }

    /// 指定したディレクトリで`golta`コマンドを実行するヘルパー
    fn golta_in_dir(&self, args: &[&str], dir: &Path) -> Output {
        Command::new(&self.golta_bin)
            .args(args)
            .current_dir(dir)
            .env("HOME", self.home_dir.path())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Failed to execute command in dir")
    }

    /// 指定したバージョンをインストールするヘルパー
    fn install_version(&mut self, version: &str) {
        let tool_version = format!("go@{}", version);
        let output = self.golta(&["install", &tool_version]);

        assert!(
            output.status.success(),
            "install command for {} failed: {}",
            version,
            String::from_utf8_lossy(&output.stderr)
        );
        self.installed_versions.push(tool_version);
    }
}

impl Drop for SmokeTest {
    /// テスト終了時に`uninstall`を実行してクリーンアップする
    fn drop(&mut self) {
        // `default`をクリアしてからアンインストール
        self.golta(&["default", "clear"]);

        for tool_version in &self.installed_versions {
            let output = self.golta(&["uninstall", tool_version]);
            // 失敗しても他のクリーンアップは続行する
            if !output.status.success() {
                eprintln!("Failed to uninstall {}: {}", tool_version, String::from_utf8_lossy(&output.stderr));
            }
        }
    }
}

#[test]
fn test_exec_go_version() {
    let test = SmokeTest::setup();

    // `default`を設定
    let tool_version = format!("go@{}", GO_VERSION);
    test.golta(&["default", &tool_version]);

    // `exec`でバージョンを確認
    let output = test.golta(&["exec", "go", "version"]);
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains(&format!("go version go{}", GO_VERSION)));
}

#[test]
fn test_pin_and_unpin() {
    let test = SmokeTest::setup();

    // グローバルデフォルトを設定
    let default_tool_version = format!("go@{}", GO_VERSION);
    test.golta(&["default", &default_tool_version]);

    // プロジェクト用のサブディレクトリを作成
    let project_dir = test.home_dir.path().join("my-project");
    std::fs::create_dir(&project_dir).unwrap();

    // 別のバージョンをプロジェクトにピン留め
    let pin_tool_version = format!("go@{}", GO_VERSION_ALT);
    let output = test.golta_in_dir(&["pin", &pin_tool_version], &project_dir);
    assert!(output.status.success());
    assert!(project_dir.join(".golta.json").exists());

    // プロジェクト内ではピン留めしたバージョンが使われることを確認
    let output = test.golta_in_dir(&["exec", "go", "version"], &project_dir);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(&format!("go version go{}", GO_VERSION_ALT)));

    // ピン留めを解除
    let output = test.golta_in_dir(&["unpin"], &project_dir);
    assert!(output.status.success());
    assert!(!project_dir.join(".golta.json").exists());

    // プロジェクト内でもグローバルデフォルトに戻ることを確認
    let output = test.golta_in_dir(&["exec", "go", "version"], &project_dir);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(&format!("go version go{}", GO_VERSION)));
}

#[test]
fn test_which_shows_correct_path() {
    let test = SmokeTest::setup();

    // `default`を設定
    let tool_version = format!("go@{}", GO_VERSION);
    test.golta(&["default", &tool_version]);

    // `which`でパスを確認
    let output = test.golta(&["which", "go"]);
    let stdout = String::from_utf8(output.stdout).unwrap();

    let expected_path_fragment = test.home_dir.path()
        .join(".golta")
        .join("versions")
        .join(GO_VERSION);

    assert!(stdout.contains(expected_path_fragment.to_str().unwrap()));
}

#[test]
fn test_cannot_uninstall_default_version() {
    let test = SmokeTest::setup();

    let tool_version = format!("go@{}", GO_VERSION);
    test.golta(&["default", &tool_version]);

    let output = test.golta(&["uninstall", &tool_version]);
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("because it is the default version"));
}