use assert_cmd::cargo::cargo_bin_cmd;
use std::path::{Path, PathBuf};

fn stdout_of(assert: &assert_cmd::assert::Assert) -> String {
    String::from_utf8_lossy(&assert.get_output().stdout)
        .trim_end_matches('\n')
        .to_string()
}

#[test]
fn windows_root_and_prefix_templates_cannot_override_tmpdir() {
    let dir = tempfile::tempdir().unwrap();
    for template in [r"C:name.XXXXXX", r"\no-such-mktemp-parent\name.XXXXXX"] {
        cargo_bin_cmd!("mktemp")
            .current_dir(dir.path())
            .arg("-p")
            .arg(dir.path())
            .arg(template)
            .assert()
            .failure()
            .stderr(predicates::str::contains("invalid template"));
    }
}

#[test]
fn backslash_suffixes_are_rejected_before_creation() {
    let dir = tempfile::tempdir().unwrap();
    for args in [
        vec![r"--suffix=\suffix", "tmp.XXXXXX"],
        vec![r"tmp.XXXXXX\suffix"],
    ] {
        cargo_bin_cmd!("mktemp")
            .current_dir(dir.path())
            .args(args)
            .assert()
            .failure()
            .stderr(predicates::str::contains("invalid suffix"));
    }
}

#[test]
fn t_option_uses_the_windows_temp_directory() {
    let dir = tempfile::tempdir().unwrap();
    let result = cargo_bin_cmd!("mktemp")
        .env("TEMP", dir.path())
        .env("TMP", dir.path())
        .args(["-t", "sample.XXXXXX"])
        .assert()
        .success();
    let created = PathBuf::from(stdout_of(&result));
    assert!(created.is_file());
    assert_eq!(created.parent().unwrap(), dir.path());
}

#[test]
fn creates_file_and_prints_its_path() {
    let dir = tempfile::tempdir().unwrap();
    let assert = cargo_bin_cmd!("mktemp")
        .args(["pre.XXXXXXXXXX"])
        .current_dir(dir.path())
        .assert()
        .success()
        .code(0);

    let printed = stdout_of(&assert);
    let created = dir.path().join(&printed);
    assert!(created.is_file(), "printed path must exist: {}", printed);

    // The prefix is kept verbatim; exactly the 10 Xs become random alnum.
    assert!(
        printed.starts_with("pre."),
        "prefix must be kept: {}",
        printed
    );
    let random_part = &printed["pre.".len()..];
    assert_eq!(random_part.len(), 10);
    assert!(
        random_part.chars().all(|c| c.is_ascii_alphanumeric()),
        "random part must be alphanumeric: {}",
        random_part
    );
}

#[test]
fn directory_option_creates_directory() {
    let dir = tempfile::tempdir().unwrap();
    let assert = cargo_bin_cmd!("mktemp")
        .args(["-d", "dir.XXXXXX"])
        .current_dir(dir.path())
        .assert()
        .success();

    let printed = stdout_of(&assert);
    let created = dir.path().join(&printed);
    assert!(
        created.is_dir(),
        "printed path must be a directory: {}",
        printed
    );
    assert!(printed.starts_with("dir."));
}

#[test]
fn no_template_uses_tmp_prefix_in_temp_dir() {
    let assert = cargo_bin_cmd!("mktemp").assert().success();

    let printed = stdout_of(&assert);
    let path = Path::new(&printed);
    assert!(path.is_file(), "printed path must exist: {}", printed);
    assert_eq!(
        path.parent(),
        Some(std::env::temp_dir().as_path()),
        "default location must be %TEMP%: {}",
        printed
    );
    let name = path.file_name().unwrap().to_string_lossy();
    assert!(name.starts_with("tmp."), "default name: {}", name);
    let random_part = &name["tmp.".len()..];
    assert_eq!(random_part.len(), 10);
    assert!(random_part.chars().all(|c| c.is_ascii_alphanumeric()));
}

#[test]
fn p_option_places_result_in_given_directory() {
    let dir = tempfile::tempdir().unwrap();
    let assert = cargo_bin_cmd!("mktemp")
        .args(["-p"])
        .arg(dir.path())
        .arg("t.XXXXXX")
        .assert()
        .success();

    let printed = stdout_of(&assert);
    let path = Path::new(&printed);
    assert!(path.is_file(), "printed path must exist: {}", printed);
    assert_eq!(path.parent(), Some(dir.path()));
}

#[test]
fn suffix_is_appended_after_random_part() {
    let dir = tempfile::tempdir().unwrap();
    let assert = cargo_bin_cmd!("mktemp")
        .args(["suf.XXXXXX", "--suffix", ".txt"])
        .current_dir(dir.path())
        .assert()
        .success();

    let printed = stdout_of(&assert);
    let created = dir.path().join(&printed);
    assert!(created.is_file(), "printed path must exist: {}", printed);
    assert!(printed.starts_with("suf."));
    assert!(printed.ends_with(".txt"));
    let random_part = &printed["suf.".len()..printed.len() - ".txt".len()];
    assert_eq!(random_part.len(), 6);
    assert!(random_part.chars().all(|c| c.is_ascii_alphanumeric()));
}

#[test]
fn suffix_must_not_contain_directory_separator() {
    cargo_bin_cmd!("mktemp")
        .args(["aXXX", "--suffix", "b/c"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(
            "invalid suffix 'b/c', contains directory separator",
        ));
}

#[test]
fn suffix_requires_template_to_end_in_x() {
    cargo_bin_cmd!("mktemp")
        .args(["foo.txt", "--suffix", ".log"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(
            "with --suffix, template 'foo.txt' must end in X",
        ));
}

#[test]
fn implicit_suffix_after_last_x_run() {
    let dir = tempfile::tempdir().unwrap();
    let assert = cargo_bin_cmd!("mktemp")
        .args(["head.XXXXtail"])
        .current_dir(dir.path())
        .assert()
        .success();

    let printed = stdout_of(&assert);
    let created = dir.path().join(&printed);
    assert!(created.is_file(), "printed path must exist: {}", printed);
    assert!(printed.starts_with("head."));
    assert!(printed.ends_with("tail"));
    let random_part = &printed["head.".len()..printed.len() - "tail".len()];
    assert_eq!(random_part.len(), 4);
    assert!(random_part.chars().all(|c| c.is_ascii_alphanumeric()));
}

#[test]
fn too_few_xs_in_template_fails() {
    cargo_bin_cmd!("mktemp")
        .arg("foo")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(
            "mktemp: too few X's in template 'foo'",
        ));
}

#[test]
fn missing_parent_directory_fails_and_reports() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("no-such-subdir");

    cargo_bin_cmd!("mktemp")
        .args(["-p"])
        .arg(&missing)
        .arg("t.XXXXXX")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(
            "mktemp: failed to create file via template",
        ));
}

#[test]
fn quiet_suppresses_creation_failure_diagnostics() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("no-such-subdir");

    cargo_bin_cmd!("mktemp")
        .args(["-q", "-p"])
        .arg(&missing)
        .arg("t.XXXXXX")
        .assert()
        .failure()
        .code(1)
        .stderr("");
}

#[test]
fn quiet_does_not_suppress_template_errors() {
    cargo_bin_cmd!("mktemp")
        .args(["-q", "foo"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("too few X's in template"));
}

#[test]
fn directory_failure_uses_directory_wording() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("no-such-subdir");

    cargo_bin_cmd!("mktemp")
        .args(["-d", "-p"])
        .arg(&missing)
        .arg("t.XXXXXX")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(
            "mktemp: failed to create directory via template",
        ));
}

#[test]
fn two_templates_are_rejected() {
    cargo_bin_cmd!("mktemp")
        .args(["a.XXXX", "b.XXXX"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("mktemp: too many templates"));
}

#[test]
fn names_differ_across_invocations() {
    let dir = tempfile::tempdir().unwrap();
    let mut seen: Vec<PathBuf> = Vec::new();
    for _ in 0..8 {
        let assert = cargo_bin_cmd!("mktemp")
            .args(["-p"])
            .arg(dir.path())
            .arg("uniq.XXXXXXXXXX")
            .assert()
            .success();
        let printed = stdout_of(&assert);
        let path = dir.path().join(&printed);
        assert!(path.is_file());
        assert!(!seen.contains(&path), "names must not repeat: {}", printed);
        seen.push(path);
    }
    assert_eq!(seen.len(), 8);
}
