use assert_cmd::cargo::cargo_bin_cmd;
use std::process::Output;

fn awk(args: &[&str], stdin: &[u8]) -> Output {
    cargo_bin_cmd!("awk")
        .args(args)
        .write_stdin(stdin.to_vec())
        .output()
        .unwrap()
}

#[test]
fn write_then_read_back_through_one_coprocess() {
    // Windows native filters stream only CRLF-terminated lines, so printf is
    // used here instead of print (which emits LF like GNU awk).
    let out = awk(
        &[
            "BEGIN { printf \"hello\\r\\n\" |& \"more\"; r = (\"more\" |& getline line); print r, line; print close(\"more\") }",
        ],
        b"",
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "1 hello\n0\n");
}

#[test]
fn multiple_messages_reuse_the_same_child() {
    let out = awk(
        &[
            "BEGIN { printf \"a\\r\\n\" |& \"more\"; printf \"b\\r\\n\" |& \"more\"; (\"more\" |& getline x); (\"more\" |& getline y); print x, y; close(\"more\") }",
        ],
        b"",
    );
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "a b\n");
}

#[test]
fn getline_into_variable_keeps_record() {
    let out = awk(
        &[
            "{ printf \"hi\\r\\n\" |& \"more\"; (\"more\" |& getline v); print $0, NF, v; close(\"more\") }",
        ],
        b"p q\n",
    );
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "p q 2 hi\n");
}

#[test]
fn getline_without_variable_installs_a_record() {
    let out = awk(
        &["BEGIN { (\"echo p q\" |& getline); print NF, $1, $2 }"],
        b"",
    );
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "2 p q\n");
}

#[test]
fn eof_returns_zero() {
    let out = awk(
        &["BEGIN { print (\"echo one\" |& getline line); print (\"echo one\" |& getline line) }"],
        b"",
    );
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "1\n0\n");
}

#[test]
fn close_returns_status_and_restarts() {
    let out = awk(
        &[
            "BEGIN { printf \"first\\r\\n\" |& \"more\"; (\"more\" |& getline a); print a; print close(\"more\"); printf \"second\\r\\n\" |& \"more\"; (\"more\" |& getline b); print b; close(\"more\") }",
        ],
        b"",
    );
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "first\n0\nsecond\n");
}

#[test]
fn close_reports_command_exit_status() {
    let out = awk(
        &["BEGIN { (\"exit 3\" |& getline x); print close(\"exit 3\") }"],
        b"",
    );
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "3\n");
}

#[test]
fn missing_command_sets_errno_and_returns_minus_one() {
    let out = awk(
        &[
            "BEGIN { r = (\"definitely_missing_cmd_xyz_123\" |& getline x); print r; print (ERRNO != \"\") }",
        ],
        b"",
    );
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "-1\n1\n");
}

#[test]
fn coprocess_and_plain_pipes_do_not_interfere() {
    // A one-way output pipe and a coprocess with separate command strings
    // keep separate children. The coprocess here is EOF-driven (echo), which
    // needs no CRLF interaction.
    let out = awk(
        &[
            "BEGIN { print \"z\" | \"findstr z\"; (\"echo y\" |& getline l); print \"got\", l; close(\"findstr z\") }",
        ],
        b"",
    );
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("got y"), "stdout: {text}");
    assert!(text.contains('z'), "stdout: {text}");
}
