use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn unknown_option_prints_as_operand() {
    // GNU: `echo -x` outputs "-x".
    cargo_bin_cmd!("echo")
        .args(["-x"])
        .assert()
        .success()
        .stdout("-x\n");
}

#[test]
fn double_dash_prints_verbatim() {
    // GNU echo does not treat -- as an end-of-options marker.
    cargo_bin_cmd!("echo")
        .args(["--"])
        .assert()
        .success()
        .stdout("--\n");
}

#[test]
fn option_clusters_apply() {
    let out = cargo_bin_cmd!("echo")
        .args(["-ne", "a\tb\n"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"a\tb\n");
}

#[test]
fn options_stop_at_first_non_option() {
    // The leading -n applies; the later -x is an operand.
    let out = cargo_bin_cmd!("echo").args(["-n", "-x"]).output().unwrap();
    assert_eq!(out.stdout, b"-x");
}

#[test]
fn e_enables_full_escape_set() {
    let out = cargo_bin_cmd!("echo")
        .args([
            "-e",
            "a\\ab\\be\\ec\\ff\\n1\\r2\\t3\\v4\\x41\\101\\'\\?\\\\",
        ])
        .output()
        .unwrap();
    assert_eq!(
        out.stdout,
        vec![
            b'a', 0x07, b'b', 0x08, b'e', 0x1B, b'c', 0x0C, b'f', b'\n', b'1', b'\r', b'2', b'\t',
            b'3', 0x0B, b'4', b'A', b'A', b'\'', b'?', b'\\', b'\n'
        ]
    );
}

#[test]
fn stop_character_suppresses_output_and_newline() {
    // GNU: \c ends output entirely; nothing after it is printed and no
    // trailing newline is added even without -n.
    let out = cargo_bin_cmd!("echo")
        .args(["-e", "c\\cd"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"c");

    let out = cargo_bin_cmd!("echo")
        .args(["-e", "x\\cy"])
        .output()
        .unwrap();
    assert_eq!(out.stdout, b"x");
}

#[test]
fn e_flag_forced_off_by_capital_e() {
    // coreutils: -E disables escape interpretation even after -e.
    let out = cargo_bin_cmd!("echo")
        .args(["-e", "-E", "a\\tb"])
        .output()
        .unwrap();
    assert_eq!(out.stdout, b"a\\tb\n");
}

#[test]
fn version_and_help_are_special() {
    let out = cargo_bin_cmd!("echo").arg("--version").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&out.stdout).contains("echo"));

    let out = cargo_bin_cmd!("echo").arg("--help").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&out.stdout).contains("Usage"));
}

#[test]
fn dash_alone_is_an_operand() {
    let out = cargo_bin_cmd!("echo").args(["-", "x"]).output().unwrap();
    assert_eq!(out.stdout, b"- x\n");
}
