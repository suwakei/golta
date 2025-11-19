use super::common::{SmokeTest, GO_VERSION};

#[test]
fn test_exec_go_version() {
    let test = SmokeTest::setup();

    // Set the `default` version
    let tool_version = format!("go@{}", GO_VERSION);
    test.golta(&["default", &tool_version]);

    // Check the version with `exec`
    let output = test.golta(&["exec", "go", "version"]);
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains(&format!("go version go{}", GO_VERSION)));
}
