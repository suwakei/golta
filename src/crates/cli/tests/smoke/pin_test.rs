use super::common::{SmokeTest, GO_VERSION, GO_VERSION_ALT};

#[test]
fn test_pin_and_unpin() {
    let test = SmokeTest::setup();

    // Set the global default
    let default_tool_version = format!("go@{}", GO_VERSION);
    test.golta(&["default", &default_tool_version]);

    // Create a subdirectory for the project
    let project_dir = test.home_dir().path().join("my-project");
    std::fs::create_dir(&project_dir).unwrap();

    // Pin a different version to the project
    let pin_tool_version = format!("go@{}", GO_VERSION_ALT);
    let output = test.golta_in_dir(&["pin", &pin_tool_version], &project_dir);
    assert!(output.status.success());
    assert!(project_dir.join(".golta.json").exists());

    // Confirm that the pinned version is used within the project
    let output = test.golta_in_dir(&["exec", "go", "version"], &project_dir);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(&format!("go version go{}", GO_VERSION_ALT)));

    // Unpin the version
    let output = test.golta_in_dir(&["unpin"], &project_dir);
    assert!(output.status.success());
    assert!(!project_dir.join(".golta.json").exists());

    // Confirm that it reverts to the global default within the project
    let output = test.golta_in_dir(&["exec", "go", "version"], &project_dir);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(&format!("go version go{}", GO_VERSION)));
}
