#[cfg(target_os = "windows")]
pub const OS: &str = "windows-amd64";
#[cfg(target_os = "linux")]
pub const OS: &str = "linux-amd64";
#[cfg(target_os = "macos")]
pub const OS: &str = "darwin-amd64";
