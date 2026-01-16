use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn run(tool: String, args: Vec<String>) {
    let env = RealGoRunEnvironment;
    let mut runner = ProcessGoRunner;

    match run_go(&tool, &args, &env, &mut runner) {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_go(
    tool: &str,
    args: &[String],
    env: &impl GoRunEnvironment,
    runner: &mut impl GoCommandRunner,
) -> Result<i32, Box<dyn Error>> {
    let version = tool
        .strip_prefix("go@")
        .ok_or("Only Go run is supported currently. Use format 'go@<version>'.")?;

    if version.is_empty() {
        return Err("Only Go run is supported currently. Use format 'go@<version>'.".into());
    }

    let go_path = env.go_binary_path(version)?;
    if !env.binary_exists(&go_path) {
        return Err(format!(
            "Go version {} is not installed. Please install it first with `golta install go@{}`.",
            version, version
        )
        .into());
    }
    runner.run(&go_path, args)
}

trait GoRunEnvironment {
    fn go_binary_path(&self, version: &str) -> Result<PathBuf, Box<dyn Error>>;
    fn binary_exists(&self, path: &Path) -> bool;
}

struct RealGoRunEnvironment;

impl GoRunEnvironment for RealGoRunEnvironment {
    fn go_binary_path(&self, version: &str) -> Result<PathBuf, Box<dyn Error>> {
        let go_executable_name = if cfg!(windows) { "go.exe" } else { "go" };
        let home = home::home_dir().ok_or("Could not find home directory")?;
        Ok(home
            .join(".golta")
            .join("versions")
            .join(version)
            .join("go")
            .join("bin")
            .join(go_executable_name))
    }

    fn binary_exists(&self, path: &Path) -> bool {
        path.exists()
    }
}

trait GoCommandRunner {
    fn run(&mut self, go_path: &Path, args: &[String]) -> Result<i32, Box<dyn Error>>;
}

struct ProcessGoRunner;

impl GoCommandRunner for ProcessGoRunner {
    fn run(&mut self, go_path: &Path, args: &[String]) -> Result<i32, Box<dyn Error>> {
        let status = Command::new(go_path).args(args).status()?;
        Ok(status.code().unwrap_or(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct MockEnv {
        go_path: PathBuf,
        requested_version: RefCell<Option<String>>,
    }

    impl MockEnv {
        fn new(go_path: &str) -> Self {
            Self {
                go_path: PathBuf::from(go_path),
                requested_version: RefCell::new(None),
            }
        }
    }

    impl GoRunEnvironment for MockEnv {
        fn go_binary_path(&self, version: &str) -> Result<PathBuf, Box<dyn Error>> {
            self.requested_version.replace(Some(version.to_string()));
            Ok(self.go_path.clone())
        }

        fn binary_exists(&self, _path: &Path) -> bool {
            true
        }
    }

    struct MockRunner {
        last_path: Option<PathBuf>,
        last_args: Vec<String>,
        exit_code: i32,
    }

    impl MockRunner {
        fn new(exit_code: i32) -> Self {
            Self {
                last_path: None,
                last_args: Vec::new(),
                exit_code,
            }
        }
    }

    impl GoCommandRunner for MockRunner {
        fn run(&mut self, go_path: &Path, args: &[String]) -> Result<i32, Box<dyn Error>> {
            self.last_path = Some(go_path.to_path_buf());
            self.last_args = args.to_vec();
            Ok(self.exit_code)
        }
    }

    #[test]
    fn errors_when_tool_is_not_go_with_version() {
        let env = MockEnv::new("/tmp/go");
        let mut runner = MockRunner::new(0);

        let result = run_go("python@3.12", &[], &env, &mut runner);

        assert!(result.is_err());
    }

    #[test]
    fn errors_when_version_missing() {
        let env = MockEnv::new("/tmp/go");
        let mut runner = MockRunner::new(0);

        let result = run_go("go@", &[], &env, &mut runner);

        assert!(result.is_err());
    }

    #[test]
    fn passes_version_and_args_to_runner() {
        let env = MockEnv::new("/tmp/go/bin/go");
        let mut runner = MockRunner::new(42);
        let args = vec!["test".into(), "./...".into()];

        let code = run_go("go@1.22.1", &args, &env, &mut runner).unwrap();

        assert_eq!(code, 42);
        assert_eq!(
            env.requested_version.borrow().as_deref(),
            Some("1.22.1"),
            "version should be forwarded without go@ prefix"
        );
        assert_eq!(runner.last_path.unwrap(), PathBuf::from("/tmp/go/bin/go"));
        assert_eq!(runner.last_args, args);
    }
}
