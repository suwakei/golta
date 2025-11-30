use serde_json::Value;
use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

/// ファイルシステムへの依存を抽象化してテストで差し替えられるようにする
pub trait PinFileSystem {
    fn exists(&self, path: &Path) -> bool;
    fn read_to_string(&self, path: &Path) -> Result<String, Box<dyn Error>>;
}

struct StdFs;

impl PinFileSystem for StdFs {
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn read_to_string(&self, path: &Path) -> Result<String, Box<dyn Error>> {
        Ok(fs::read_to_string(path)?)
    }
}

/// カレントディレクトリから親方向に `.golta.json` を探し、Go バージョンとパスを返す
pub fn find_pinned_go_version() -> Result<Option<(String, PathBuf)>, Box<dyn Error>> {
    let fs = StdFs;
    let start = env::current_dir()?;
    find_pinned_go_version_from(&fs, start.as_path())
}

/// 任意の開始ディレクトリとファイルシステム実装を指定して検索（テスト向け）
pub fn find_pinned_go_version_from<F: PinFileSystem>(
    fs: &F,
    start_dir: &Path,
) -> Result<Option<(String, PathBuf)>, Box<dyn Error>> {
    let mut current_dir = start_dir.to_path_buf();
    loop {
        let pin_file_path = current_dir.join(".golta.json");
        if fs.exists(&pin_file_path) {
            let content = fs.read_to_string(&pin_file_path)?;
            if let Some(go_ver) = extract_go_version(&content)? {
                return Ok(Some((go_ver, pin_file_path)));
            }
        }

        if !current_dir.pop() {
            return Ok(None);
        }
    }
}

fn extract_go_version(raw_json: &str) -> Result<Option<String>, Box<dyn Error>> {
    let json: Value = serde_json::from_str(raw_json)?;
    Ok(json
        .get("go")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockFs {
        files: HashMap<PathBuf, String>,
    }

    impl MockFs {
        fn new(files: HashMap<PathBuf, String>) -> Self {
            Self { files }
        }
    }

    impl PinFileSystem for MockFs {
        fn exists(&self, path: &Path) -> bool {
            self.files.contains_key(path)
        }

        fn read_to_string(&self, path: &Path) -> Result<String, Box<dyn Error>> {
            self.files
                .get(path)
                .cloned()
                .ok_or_else(|| "missing file".into())
        }
    }

    fn pin(path: &str, go: &str) -> (PathBuf, String) {
        (PathBuf::from(path), format!("{{\"go\":\"{}\"}}", go))
    }

    #[test]
    fn finds_in_current_dir() {
        let start = PathBuf::from("project/sub");
        let pin_path = PathBuf::from("project/sub/.golta.json");
        let fs = MockFs::new(HashMap::from([pin(&pin_path.to_string_lossy(), "1.22.0")]));

        let found = find_pinned_go_version_from(&fs, &start).unwrap();

        assert_eq!(
            found,
            Some(("1.22.0".to_string(), pin_path)),
            "should pick current directory pin"
        );
    }

    #[test]
    fn climbs_to_parent_when_missing_locally() {
        let start = PathBuf::from("project/sub");
        let parent_pin = PathBuf::from("project/.golta.json");
        let fs = MockFs::new(HashMap::from([pin(
            &parent_pin.to_string_lossy(),
            "1.21.1",
        )]));

        let found = find_pinned_go_version_from(&fs, &start).unwrap();

        assert_eq!(
            found,
            Some(("1.21.1".to_string(), parent_pin)),
            "should find pin in ancestor"
        );
    }

    #[test]
    fn returns_none_when_no_pin_anywhere() {
        let start = PathBuf::from("project/sub");
        let fs = MockFs::new(HashMap::new());

        let found = find_pinned_go_version_from(&fs, &start).unwrap();

        assert!(found.is_none());
    }

    #[test]
    fn ignores_pin_without_go_key_and_continues_search() {
        let start = PathBuf::from("project/sub");
        let invalid_pin_path = PathBuf::from("project/sub/.golta.json");
        let valid_pin_path = PathBuf::from("project/.golta.json");
        let fs = MockFs::new(HashMap::from([
            (invalid_pin_path.clone(), "{}".to_string()),
            pin(&valid_pin_path.to_string_lossy(), "1.20.0"),
        ]));
        let found = find_pinned_go_version_from(&fs, &start).unwrap();

        assert_eq!(
            found,
            Some(("1.20.0".to_string(), valid_pin_path)),
            "should skip files without go key"
        );
    }

    #[test]
    fn propagates_parse_error_on_invalid_json() {
        let start = PathBuf::from("project");
        let pin_path = PathBuf::from("project/.golta.json");
        let fs = MockFs::new(HashMap::from([(pin_path, "not json".to_string())]));

        let err = find_pinned_go_version_from(&fs, &start).unwrap_err();

        assert!(
            err.downcast_ref::<serde_json::Error>().is_some(),
            "expected serde_json parse error, got {}",
            err
        );
    }
}
