use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq)]
pub enum UnpinOutcome {
    Removed,
    NotPinned,
}

pub trait PinFileSystem {
    fn exists(&self, path: &Path) -> bool;
    fn remove_file(&self, path: &Path) -> Result<(), Box<dyn Error>>;
}

struct StdFs;

impl PinFileSystem for StdFs {
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn remove_file(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        fs::remove_file(path)?;
        Ok(())
    }
}

pub fn run() {
    let fs = StdFs;
    match current_pin_file().and_then(|pin_file| unpin(&fs, pin_file.as_path())) {
        Ok(UnpinOutcome::Removed) => println!("Removed pinned version for this project."),
        Ok(UnpinOutcome::NotPinned) => println!("No Go version is pinned in this directory."),
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn current_pin_file() -> Result<PathBuf, Box<dyn Error>> {
    Ok(env::current_dir()?.join(".golta.json"))
}

pub fn unpin<F: PinFileSystem>(fs: &F, pin_file: &Path) -> Result<UnpinOutcome, Box<dyn Error>> {
    if fs.exists(pin_file) {
        fs.remove_file(pin_file)?;
        return Ok(UnpinOutcome::Removed);
    }

    Ok(UnpinOutcome::NotPinned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct MockFs {
        files: RefCell<Vec<PathBuf>>,
    }

    impl MockFs {
        fn new(files: Vec<PathBuf>) -> Self {
            Self {
                files: RefCell::new(files),
            }
        }
    }

    impl PinFileSystem for MockFs {
        fn exists(&self, path: &Path) -> bool {
            self.files.borrow().iter().any(|p| p == path)
        }

        fn remove_file(&self, path: &Path) -> Result<(), Box<dyn Error>> {
            let mut files = self.files.borrow_mut();
            if let Some(pos) = files.iter().position(|p| p == path) {
                files.remove(pos);
                Ok(())
            } else {
                Err("file missing".into())
            }
        }
    }

    #[test]
    fn removes_when_present() {
        let pin = PathBuf::from(".golta.json");
        let fs = MockFs::new(vec![pin.clone()]);

        let result = unpin(&fs, &pin).unwrap();

        assert_eq!(result, UnpinOutcome::Removed);
        assert!(!fs.exists(&pin));
    }

    #[test]
    fn reports_not_pinned_when_absent() {
        let pin = PathBuf::from(".golta.json");
        let fs = MockFs::new(vec![]);

        let result = unpin(&fs, &pin).unwrap();

        assert_eq!(result, UnpinOutcome::NotPinned);
    }
}
