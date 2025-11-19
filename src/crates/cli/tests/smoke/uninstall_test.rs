use super::common::{SmokeTest, GO_VERSION};

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
