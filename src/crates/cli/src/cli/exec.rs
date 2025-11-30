use crate::shared::active_version::find_active_go_version;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn run(tool: String, args: Vec<String>) {
    let env = RealGoEnvironment;
    let mut runner = ProcessGoRunner;

    match exec_go(&tool, &args, &env, &mut runner) {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn exec_go(
    tool: &str,
    args: &[String],
    env: &impl GoEnvironment,
    runner: &mut impl GoCommandRunner,
) -> Result<i32, Box<dyn Error>> {
    if tool != "go" {
        return Err("Only `go` is supported for exec currently.".into());
    }

    let version_str = env
        .active_version()?
        .ok_or("No Go version is active. Use `golta pin` or `golta default`.")?;
    let version = version_str.trim();
    let version = version.trim_start_matches("go@");

    if version.is_empty() {
        return Err("No Go version is active. Use `golta pin` or `golta default`.".into());
    }

    let go_path = env.go_binary_path(version)?;
    runner.run(&go_path, args)
}

trait GoEnvironment {
    fn active_version(&self) -> Result<Option<String>, Box<dyn Error>>;
    fn go_binary_path(&self, version: &str) -> Result<PathBuf, Box<dyn Error>>;
}

struct RealGoEnvironment;

impl GoEnvironment for RealGoEnvironment {
    fn active_version(&self) -> Result<Option<String>, Box<dyn Error>> {
        find_active_go_version()
    }

    fn go_binary_path(&self, version: &str) -> Result<PathBuf, Box<dyn Error>> {
        let go_executable_name = if cfg!(windows) { "go.exe" } else { "go" };
        let home = home::home_dir().ok_or("Could not find home directory")?;
        Ok(home
            .join(".golta")
            .join("versions")
            .join(version.trim_start_matches("go@"))
            .join("bin")
            .join(go_executable_name))
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
        active_version: Option<String>,
        go_path: PathBuf,
        requested_version: RefCell<Option<String>>,
    }

    impl MockEnv {
        fn new(active_version: Option<&str>, go_path: &str) -> Self {
            Self {
                active_version: active_version.map(ToString::to_string),
                go_path: PathBuf::from(go_path),
                requested_version: RefCell::new(None),
            }
        }
    }

    impl GoEnvironment for MockEnv {
        fn active_version(&self) -> Result<Option<String>, Box<dyn Error>> {
            Ok(self.active_version.clone())
        }

        fn go_binary_path(&self, version: &str) -> Result<PathBuf, Box<dyn Error>> {
            self.requested_version.replace(Some(version.to_string()));
            Ok(self.go_path.clone())
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
    fn errors_when_tool_is_not_go() {
        let env = MockEnv::new(Some("go@1.22.1"), "/tmp/go/bin/go");
        let mut runner = MockRunner::new(0);

        let result = exec_go("python", &[], &env, &mut runner);

        assert!(result.is_err());
    }

    #[test]
    fn errors_when_no_active_version() {
        let env = MockEnv::new(None, "/tmp/go/bin/go");
        let mut runner = MockRunner::new(0);

        let result = exec_go("go", &[], &env, &mut runner);

        assert!(result.is_err());
    }

    #[test]
    fn passes_clean_version_and_runs_command() {
        let env = MockEnv::new(Some("go@1.22.1"), "/tmp/go/bin/go");
        let mut runner = MockRunner::new(42);
        let args = vec!["fmt".to_string(), "./...".to_string()];

        let code = exec_go("go", &args, &env, &mut runner).unwrap();

        assert_eq!(code, 42);
        assert_eq!(env.requested_version.borrow().as_deref(), Some("1.22.1"));
        assert_eq!(runner.last_path.unwrap(), PathBuf::from("/tmp/go/bin/go"));
        assert_eq!(runner.last_args, args);
    }
}
