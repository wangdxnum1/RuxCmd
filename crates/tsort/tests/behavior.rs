use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn sorts_a_simple_chain() {
    cargo_bin_cmd!("tsort")
        .write_stdin(b"a b\nb c\n")
        .assert()
        .success()
        .stdout("a\nb\nc\n");
}

#[test]
fn odd_tokens_include_the_input_name() {
    // GNU: "tsort: -: input contains an odd number of tokens", exit 1.
    cargo_bin_cmd!("tsort")
        .write_stdin(b"a\n")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(
            "tsort: -: input contains an odd number of tokens",
        ));
}

#[test]
fn gnu_reference_chain() {
    // GNU tsort on "a\nb\n" prints a then b.
    cargo_bin_cmd!("tsort")
        .write_stdin(b"a\nb\n")
        .assert()
        .success()
        .stdout("a\nb\n");
}

#[test]
fn odd_token_count_is_an_error() {
    cargo_bin_cmd!("tsort")
        .write_stdin(b"a b c\n")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("odd number of tokens"));
}

#[test]
fn cycle_matches_gnu_output() {
    // GNU: loop members go to stderr AND stdout; exit 1.
    let out = cargo_bin_cmd!("tsort")
        .write_stdin(b"a b\nb a\n")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "a\nb\n");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("input contains a loop:"), "stderr: {err}");
}

#[test]
fn duplicate_edges_do_not_double_count() {
    cargo_bin_cmd!("tsort")
        .write_stdin(b"a b\na b\nb c\n")
        .assert()
        .success()
        .stdout("a\nb\nc\n");
}

use predicates::prelude::*;
