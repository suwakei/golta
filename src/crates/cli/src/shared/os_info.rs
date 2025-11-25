/// Returns the OS and architecture string for the current target,
/// and the appropriate archive format (`zip` or `tar.gz`).
pub const fn get_os_arch_and_format() -> (&'static str, &'static str) {
    // For x86_64 architecture
    if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        ("windows-amd64", "zip")
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        ("linux-amd64", "tar.gz")
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        ("darwin-amd64", "tar.gz")
    // For ARM64/AArch64 architecture
    } else if cfg!(all(target_os = "windows", target_arch = "aarch64")) {
        ("windows-arm64", "zip")
    } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
        ("linux-arm64", "tar.gz")
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        ("darwin-arm64", "tar.gz")
    } else {
        // This will cause a compile-time error for unsupported targets.
        panic!("Unsupported OS/architecture combination")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::consts::{ARCH, OS as OS_NAME};

    #[test]
    fn test_get_os_arch_for_current_target() {
        let expected_tuple = match (OS_NAME, ARCH) {
            ("windows", "x86_64") => ("windows-amd64", "zip"),
            ("linux", "x86_64") => ("linux-amd64", "tar.gz"),
            ("macos", "x86_64") => ("darwin-amd64", "tar.gz"),
            ("windows", "aarch64") => ("windows-arm64", "zip"),
            ("linux", "aarch64") => ("linux-arm64", "tar.gz"),
            ("macos", "aarch64") => ("darwin-arm64", "tar.gz"),
            _ => panic!(
                "Running tests on unsupported OS/architecture: {}-{}",
                OS_NAME, ARCH
            ),
        };

        assert_eq!(get_os_arch_and_format(), expected_tuple);
    }
}
