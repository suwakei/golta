use super::common::{SmokeTest, GO_VERSION};

#[test]
fn test_which_shows_correct_path() {
    let test = SmokeTest::setup();

    // Set the `default` version
    let tool_version = format!("go@{}", GO_VERSION);
    test.golta(&["default", &tool_version]);

    // Check the path with `which`
    let output = test.golta(&["which", "go"]);
    let stdout = String::from_utf8(output.stdout).unwrap();

    let expected_path_fragment = test
        .home_dir()
        .path()
        .join(".golta")
        .join("versions")
        .join(GO_VERSION);

    assert!(stdout.contains(expected_path_fragment.to_str().unwrap()));
}
