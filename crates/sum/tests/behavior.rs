use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::TempDir;

#[test]
fn bsd_format_matches_gnu_vector() {
    // GNU sum on "hello\n" prints checksum 36979 with one 1K block.
    let dir = TempDir::new().unwrap();
    let f = dir.path().join("s.dat");
    fs::write(&f, b"hello\n").unwrap();
    let out = cargo_bin_cmd!("sum")
        .arg(f.to_string_lossy().into_owned())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "36979     1\n");
}

#[test]
fn sysv_format_includes_filename() {
    // GNU sum -s prints "542 1 s.dat" for the same content; the name is
    // the operand as written, so the test uses a relative path.
    let dir = TempDir::new().unwrap();
    let f = dir.path().join("s.dat");
    fs::write(&f, b"hello\n").unwrap();
    let out = cargo_bin_cmd!("sum")
        .args(["-s", "s.dat"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "542 1 s.dat\n");
}

#[test]
fn stdin_omits_filename_in_both_modes() {
    let out = cargo_bin_cmd!("sum")
        .write_stdin(b"hello\n")
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout), "36979     1\n");

    let out = cargo_bin_cmd!("sum")
        .args(["-s"])
        .write_stdin(b"hello\n")
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout), "542 1\n");
}

#[test]
fn empty_input_yields_zeroes() {
    let out = cargo_bin_cmd!("sum").write_stdin(b"").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout), "00000     0\n");
}

#[test]
fn missing_file_exits_one() {
    cargo_bin_cmd!("sum")
        .arg("definitely-missing-sum-input.bin")
        .assert()
        .failure()
        .code(1);
}
