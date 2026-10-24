//! Integration tests for test: numeric comparisons, single-argument and
//! string forms, negation, and file tests.

use std::path::PathBuf;
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_test")
}

fn code(args: &[&str]) -> Option<i32> {
    Command::new(bin())
        .args(args)
        .output()
        .unwrap()
        .status
        .code()
}

#[test]
fn numeric_comparisons() {
    assert_eq!(code(&["1", "-eq", "1"]), Some(0));
    assert_eq!(code(&["1", "-ne", "2"]), Some(0));
    assert_eq!(code(&["1", "-lt", "2"]), Some(0));
    assert_eq!(code(&["2", "-le", "2"]), Some(0));
    assert_eq!(code(&["3", "-gt", "2"]), Some(0));
    assert_eq!(code(&["2", "-ge", "3"]), Some(1));
    // Non-integer operands are usage errors.
    assert_eq!(code(&["abc", "-eq", "1"]), Some(2));
}

#[test]
fn single_argument_and_string_forms() {
    // A single non-empty argument is true; the empty string is false.
    assert_eq!(code(&["foo"]), Some(0));
    assert_eq!(code(&[""]), Some(1));

    // -z: true if the string is empty; -n: true if it is not.
    assert_eq!(code(&["-z", ""]), Some(0));
    assert_eq!(code(&["-z", "x"]), Some(1));
    assert_eq!(code(&["-n", "x"]), Some(0));
    assert_eq!(code(&["-n", ""]), Some(1));

    assert_eq!(code(&["foo", "=", "foo"]), Some(0));
    assert_eq!(code(&["foo", "!=", "bar"]), Some(0));
}

#[test]
fn negation_and_file_tests() {
    assert_eq!(code(&["!", "-e", "definitely-missing-file-xyz"]), Some(0));
    assert_eq!(code(&["!", "foo"]), Some(1));

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cargo_toml = manifest.join("Cargo.toml").into_os_string();
    assert_eq!(code(&["-e", &cargo_toml.to_string_lossy()]), Some(0));
    assert_eq!(code(&["-f", &cargo_toml.to_string_lossy()]), Some(0));

    // -s is true only when the file has a non-zero size.
    let empty = std::env::temp_dir().join(format!("ruxcmd-test-empty-{}", std::process::id()));
    std::fs::write(&empty, b"").unwrap();
    assert_eq!(code(&["-s", &empty.to_string_lossy()]), Some(1));
    std::fs::write(&empty, b"x").unwrap();
    assert_eq!(code(&["-s", &empty.to_string_lossy()]), Some(0));
    let _ = std::fs::remove_file(&empty);

    // Windows approximations: -r/-x exist, -w honors the readonly bit.
    assert_eq!(code(&["-r", &cargo_toml.to_string_lossy()]), Some(0));
    assert_eq!(code(&["-w", &cargo_toml.to_string_lossy()]), Some(0));
}
