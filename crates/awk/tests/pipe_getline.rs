use assert_cmd::cargo::cargo_bin_cmd;
use std::process::Output;

fn awk(args: &[&str], stdin: &[u8]) -> Output {
    cargo_bin_cmd!("awk")
        .args(args)
        .write_stdin(stdin)
        .output()
        .unwrap()
}

#[test]
fn pipe_getline_reads_lines_into_variable() {
    let out = awk(
        &["BEGIN { while ((\"echo a&echo b\" | getline line) > 0) print line }"],
        b"",
    );
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "a\nb\n");
}

#[test]
fn pipe_getline_into_record_splits_fields() {
    let out = awk(&["BEGIN { \"echo p q\" | getline; print NF, $1, $2 }"], b"");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "2 p q\n");
}

#[test]
fn pipe_getline_into_variable_keeps_record() {
    let out = awk(&["{ \"echo hi\" | getline v; print $0, NF, v }"], b"p q\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "p q 2 hi\n");
}

#[test]
fn pipe_getline_returns_zero_at_eof() {
    let out = awk(
        &["BEGIN { print (\"echo one\" | getline line); print (\"echo one\" | getline line) }"],
        b"",
    );
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "1\n0\n");
}

#[test]
fn same_command_reuses_one_process() {
    // The second read continues the first process instead of restarting it.
    let out = awk(
        &["BEGIN { \"echo a&echo b\" | getline x; \"echo a&echo b\" | getline y; print x, y }"],
        b"",
    );
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "a b\n");
}

#[test]
fn missing_command_returns_minus_one_with_errno() {
    let out = awk(
        &[
            "BEGIN { r = (\"definitely_missing_cmd_xyz_123\" | getline line); print r; print (ERRNO != \"\") }",
        ],
        b"",
    );
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "-1\n1\n");
}

#[test]
fn failed_command_does_not_break_later_commands() {
    let out = awk(
        &[
            "BEGIN { a = (\"definitely_missing_cmd_xyz_123\" | getline x); b = (\"echo ok\" | getline y); print a, b, y }",
        ],
        b"",
    );
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "-1 1 ok\n");
}

#[test]
fn close_returns_status_and_restarts_the_pipe() {
    let out = awk(
        &[
            "BEGIN { \"echo first\" | getline a; print a; print close(\"echo first\"); \"echo first\" | getline b; print b }",
        ],
        b"",
    );
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "first\n0\nfirst\n");
}

#[test]
fn close_reports_nonzero_pipe_status() {
    let out = awk(
        &["BEGIN { \"exit 3\" | getline x; print close(\"exit 3\") }"],
        b"",
    );
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "3\n");
}

#[test]
fn close_reports_output_pipe_status() {
    let out = awk(
        &["BEGIN { print \"x\" | \"exit 4\"; print close(\"exit 4\") }"],
        b"",
    );
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "4\n");
}

#[test]
fn pipe_getline_does_not_change_nr() {
    // GNU: cmd | getline does not increment NR or FNR.
    let out = awk(
        &["{ while ((\"echo a&echo b\" | getline l) > 0) print NR, l }"],
        b"one\ntwo\n",
    );
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "1 a\n1 b\n");
}
