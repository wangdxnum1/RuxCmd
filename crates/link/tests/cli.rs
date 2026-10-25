use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use std::io::Write;

#[test]
fn creates_hard_link_with_shared_content() {
    let dir = tempfile::tempdir().unwrap();
    let orig = dir.path().join("orig.txt");
    let linked = dir.path().join("linked.txt");
    fs::write(&orig, "hello").unwrap();

    cargo_bin_cmd!("link")
        .arg(&orig)
        .arg(&linked)
        .assert()
        .success()
        .code(0)
        .stdout("")
        .stderr("");

    assert_eq!(
        fs::read_to_string(&linked).unwrap(),
        "hello",
        "the link must read the original content"
    );
    // Hard links share data: writing through one is visible via the other.
    fs::File::options()
        .append(true)
        .open(&linked)
        .unwrap()
        .write_all(b"-appended")
        .unwrap();
    assert_eq!(
        fs::read_to_string(&orig).unwrap(),
        "hello-appended",
        "the original must see writes through the link"
    );
}

#[test]
fn missing_source_fails_with_gnu_message() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("target.txt");

    cargo_bin_cmd!("link")
        .arg(dir.path().join("no-such.txt"))
        .arg(&target)
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("link: cannot create link"));
    assert!(!target.exists());
}

#[test]
fn no_operands_reports_missing_file_operand() {
    cargo_bin_cmd!("link")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("link: missing file operand"));
}

#[test]
fn one_operand_reports_missing_operand_after() {
    cargo_bin_cmd!("link")
        .arg("only-one")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("link: missing operand after"));
}

#[test]
fn three_operands_report_extra_operand() {
    cargo_bin_cmd!("link")
        .args(["a", "b", "c"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("link: extra operand 'c'"));
}
