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
fn cycle_is_reported() {
    cargo_bin_cmd!("tsort")
        .write_stdin(b"a b\nb a\n")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("cycle"));
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
