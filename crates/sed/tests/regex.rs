use assert_cmd::cargo::cargo_bin_cmd;

fn run(script: &str, stdin: &[u8]) -> (String, Option<i32>) {
    run_args(&[script], stdin)
}

fn run_args(args: &[&str], stdin: &[u8]) -> (String, Option<i32>) {
    let out = cargo_bin_cmd!("sed")
        .args(args)
        .write_stdin(stdin)
        .output()
        .unwrap();
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        out.status.code(),
    )
}

#[test]
fn basic_regex_with_dot_and_star() {
    // GNU: `sed 's/f.*o/XX/'` matches the whole run.
    let (out, code) = run("s/f.*o/XX/", b"fabco\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "XX\n");
}

#[test]
fn group_capture_in_replacement() {
    let (out, code) = run("s/\\(ab\\)\\(c\\)/\\2\\1/", b"abcd\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "cabd\n");
}

#[test]
fn ampersand_inserts_whole_match() {
    let (out, code) = run("s/[0-9][0-9]*/<&>/", b"a42b\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "a<42>b\n");
}

#[test]
fn escaped_ampersand_is_literal() {
    let (out, code) = run("s/42/\\&/", b"a42b\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "a&b\n");
}

#[test]
fn global_flag_replaces_every_match() {
    let (out, code) = run("s/o/0/g", b"foo boo\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "f00 b00\n");
}

#[test]
fn nth_occurrence_flag() {
    // GNU: `s/o/0/2` replaces only the second occurrence.
    let (out, code) = run("s/o/0/2", b"foo\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "fo0\n");
}

#[test]
fn print_flag_echoes_after_change() {
    // -n plus s///p prints only lines that changed; banana contains no `b`
    // uppercase-able... it does: b->B changes banana, so use a marker that
    // only matches the first line.
    let out = cargo_bin_cmd!("sed")
        .args(["-n", "s/^a/A/p"])
        .write_stdin(b"apple\nbanana\n")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "Apple\n");
}

#[test]
fn case_insensitive_flag() {
    let (out, code) = run("s/hello/bye/i", b"HELLO world\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "bye world\n");
}

#[test]
fn interval_expression() {
    let (out, code) = run("s/x\\{2,3\\}/Y/g", b"x xx xxx\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "x Y Y\n");
}

#[test]
fn regex_address_selects_lines() {
    let (out, code) = run("/^b/d", b"apple\nbanana\ncherry\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "apple\ncherry\n");
}

#[test]
fn invalid_bre_fails_at_startup() {
    cargo_bin_cmd!("sed")
        .arg("s/a\\(b/X/")
        .write_stdin(b"a\n")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("sed: -e expression #1"));
}

#[test]
fn back_reference_pattern_is_rejected_clearly() {
    // Documented limitation: BRE back references are unsupported.
    cargo_bin_cmd!("sed")
        .arg("s/\\(a\\)\\1/X/")
        .write_stdin(b"aa\n")
        .assert()
        .failure()
        .stderr(predicates::str::contains("back references"));
}

#[test]
fn non_utf8_bytes_survive_substitution() {
    let out = cargo_bin_cmd!("sed")
        .arg("s/b/X/")
        .write_stdin(&[0x61, 0xFF, 0x62, 0x63, b'\n'])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, vec![0x61, 0xFF, b'X', b'c', b'\n']);
}

#[test]
fn replacement_newline_escape() {
    // GNU: `\n` in a replacement emits a real newline.
    let (out, code) = run("s/,/\\n/", b"a,b\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "a\nb\n");
}

#[test]
fn escaped_delimiter_in_pattern() {
    // `\/` inside an address is a literal slash.
    let (out, code) = run("/a\\/b/d", b"x\na/b\ny\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "x\ny\n");
}

use predicates::prelude::*;

#[test]
fn ere_mode_alternation_and_intervals() {
    // -E: unescaped ERE operators keep their operator meaning.
    let (out, code) = run_args(&["-E", "s/cat|dog/PET/g"], b"cat bird dog\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "PET bird PET\n");

    let (out, code) = run_args(&["-E", "s/o{2}/OO/"], b"foot\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "fOOt\n");
}

#[test]
fn ere_mode_groups_are_unescaped() {
    // In ERE, ( ) are groups without backslashes; backrefs still rejected.
    let out = cargo_bin_cmd!("sed")
        .args(["-E", r"s/(f[o]+t)/<\1>/g"])
        .write_stdin(b"foot bar boot\n")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "<foot> bar boot\n");

    cargo_bin_cmd!("sed")
        .args(["-E", r"s/(a)\1/X/"])
        .write_stdin(b"aa\n")
        .assert()
        .failure()
        .stderr(predicates::str::contains("back references"));
}

#[test]
fn ere_mode_r_alias_and_bre_unchanged() {
    // -r is the historical alias.
    let (out, code) = run_args(&["-r", "s/o{2}/OO/"], b"foot\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "fOOt\n");

    // BRE stays literal: + is a plain plus sign without -E.
    let (out, code) = run("s/o+/0/", b"foot\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "foot\n");
}
