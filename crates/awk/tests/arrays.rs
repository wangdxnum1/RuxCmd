use assert_cmd::cargo::cargo_bin_cmd;

fn awk(args: &[&str], stdin: &[u8]) -> Output {
    cargo_bin_cmd!("awk")
        .args(args)
        .write_stdin(stdin)
        .output()
        .unwrap()
}

#[test]
fn array_assignment_and_read() {
    let out = awk(&["BEGIN { a[\"k\"] = 42; print a[\"k\"] }"], b"");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "42\n");
}

#[test]
fn split_populates_array_and_returns_count() {
    let out = awk(
        &["{ n = split($0, arr, \",\"); print n, arr[1], arr[2], arr[3] }"],
        b"a,b,c\n",
    );
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "3 a b c\n");
}

#[test]
fn split_defaults_to_fs() {
    let out = awk(&["{ split($0, arr); print arr[2] }"], b"one two three\n");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "two\n");
}

#[test]
fn in_operator_checks_membership() {
    let out = awk(
        &["BEGIN { a[\"x\"] = 1; print (\"x\" in a), (\"y\" in a) }"],
        b"",
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "1 0\n");
}

#[test]
fn delete_removes_element_and_whole_array() {
    // Two separate prints, so the results land on two lines.
    let out = awk(
        &["BEGIN { a[1] = 1; a[2] = 2; delete a[1]; print (1 in a); delete a; print (2 in a) }"],
        b"",
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "0\n0\n");
}

#[test]
fn for_in_iterates_keys() {
    // Order is unspecified in POSIX; this port sorts keys.
    let out = awk(
        &["BEGIN { a[\"b\"] = 1; a[\"a\"] = 2; for (k in a) print k }"],
        b"",
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "a\nb\n");
}

#[test]
fn compound_assignment_on_scalars_and_elements() {
    let out = awk(
        &[
            "BEGIN { s = 0; a[1] = 10; a[2] = 20; for (k in a) s += a[k]; print s; s *= 2; print s }",
        ],
        b"",
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "30\n60\n");
}

#[test]
fn classic_for_with_postfix_increment() {
    let out = awk(&["BEGIN { for (i = 0; i < 3; i++) print i }"], b"");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "0\n1\n2\n");
}

use std::process::Output;
