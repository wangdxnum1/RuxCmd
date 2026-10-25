use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn prints_a_non_empty_single_line_and_exits_zero() {
    let output = cargo_bin_cmd!("users")
        .assert()
        .success()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();

    assert!(stdout.ends_with('\n'), "output must end with a newline");
    let line = stdout.trim_end_matches('\n');
    assert!(!line.is_empty(), "user list must not be empty");
    assert!(
        !line.contains('\n') && !line.contains('\r'),
        "exactly one line expected, got: {stdout:?}"
    );
}

#[test]
fn count_flag_prints_only_a_positive_number() {
    let output = cargo_bin_cmd!("users")
        .args(["--count"])
        .assert()
        .success()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();

    let count: usize = stdout
        .trim()
        .parse()
        .expect("--count must print a single number");
    assert!(count > 0, "at least the current user must be counted");
}

#[test]
fn short_count_flag_matches_long() {
    let long = cargo_bin_cmd!("users")
        .arg("--count")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let short = cargo_bin_cmd!("users")
        .arg("-q")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(long, short, "-q and --count must agree");
}

#[test]
fn user_names_have_no_whitespace_padding() {
    // Every token on the line must be a plain name (no WTS padding), i.e.
    // splitting on whitespace yields the deduplicated list.
    let output = cargo_bin_cmd!("users")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();
    let names: Vec<&str> = stdout.split_whitespace().collect();
    assert!(!names.is_empty());
    let mut sorted = names.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(
        names.len(),
        sorted.len(),
        "duplicate user names must be removed, got: {stdout:?}"
    );
}
