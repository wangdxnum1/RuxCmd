use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn hex_dump_default_format() {
    let out = cargo_bin_cmd!("xxd")
        .write_stdin(b"hello\n")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.starts_with("00000000:"), "output: {text}");
    assert!(text.contains("6865 6c6c"), "output: {text}");
}

#[test]
fn empty_input_produces_no_output() {
    let out = cargo_bin_cmd!("xxd").write_stdin(b"").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert!(out.stdout.is_empty());
}

#[test]
fn revert_round_trips() {
    let out = cargo_bin_cmd!("xxd")
        .write_stdin(b"hello")
        .output()
        .unwrap();
    let hex_dump = String::from_utf8_lossy(&out.stdout).into_owned();

    let out = cargo_bin_cmd!("xxd")
        .args(["-r"])
        .write_stdin(hex_dump.as_bytes())
        .output()
        .unwrap();
    assert_eq!(out.stdout, b"hello");
}
