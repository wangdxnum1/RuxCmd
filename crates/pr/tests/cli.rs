use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

// Regression for P0 #28: the page-header date was derived from a hand-rolled
// seconds-since-epoch estimate (no leap years, 30-day months), and a bare `-n`
// was a clap error (`-n FILE` even swallowed the file name as the separator
// spec). GNU pr: `-n[SEP][DIGITS]`, bare `-n` = TAB separator + width 5, and
// the header shows the real local date.

fn write_input() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("in.txt");
    std::fs::write(&f, "hello\nworld\n").unwrap();
    (dir, f)
}

#[test]
fn header_shows_current_local_date() {
    let (_dir, f) = write_input();
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    cargo_bin_cmd!("pr")
        .arg(f.as_os_str())
        .assert()
        .success()
        .stdout(predicate::str::contains(today))
        .stdout(predicate::str::contains("Page 1"));
}

#[test]
fn plain_run_with_file_succeeds() {
    let (_dir, f) = write_input();
    cargo_bin_cmd!("pr")
        .arg(f.as_os_str())
        .assert()
        .success()
        .stdout(predicate::str::contains("hello"))
        .stdout(predicate::str::contains("world"));
}

#[test]
fn bare_dash_n_with_stdin_uses_tab_and_width_5() {
    cargo_bin_cmd!("pr")
        .arg("-n")
        .write_stdin("a\nb\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("    1\ta"))
        .stdout(predicate::str::contains("    2\tb"));
}

#[test]
fn bare_dash_n_does_not_swallow_file_argument() {
    let (_dir, f) = write_input();
    cargo_bin_cmd!("pr")
        .arg("-n")
        .arg(f.as_os_str())
        .assert()
        .success()
        .stdout(predicate::str::contains("    1\thello"))
        .stdout(predicate::str::contains("    2\tworld"))
        // the file name must remain the page header, not become the -n spec
        .stdout(predicate::str::contains("in.txt"));
}

#[test]
fn n_with_separator_and_width() {
    let (_dir, f) = write_input();
    cargo_bin_cmd!("pr")
        .args(["-n:5"])
        .arg(f.as_os_str())
        .assert()
        .success()
        .stdout(predicate::str::contains("    1:hello"))
        .stdout(predicate::str::contains("    2:world"));
}

#[test]
fn n_with_digits_only_sets_width() {
    let (_dir, f) = write_input();
    cargo_bin_cmd!("pr")
        .args(["-n3"])
        .arg(f.as_os_str())
        .assert()
        .success()
        .stdout(predicate::str::contains("  1\thello"));
}

#[test]
fn n_with_separator_only_defaults_width_5() {
    let (_dir, f) = write_input();
    cargo_bin_cmd!("pr")
        .args(["-n:"])
        .arg(f.as_os_str())
        .assert()
        .success()
        .stdout(predicate::str::contains("    1:hello"));
}

#[test]
fn n_invalid_digits_report_error() {
    let (_dir, f) = write_input();
    cargo_bin_cmd!("pr")
        .args(["-n:xy"])
        .arg(f.as_os_str())
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid --number-lines argument"));
}
