use assert_cmd::cargo::cargo_bin_cmd;
use std::os::windows::fs::symlink_file;

fn make_symlink(dir: &std::path::Path, link: &str, target: &str) -> bool {
    // Creating file symlinks needs Developer Mode or admin rights; hosts
    // without either legitimately skip these checks (same policy as ln).
    match symlink_file(target, dir.join(link)) {
        Ok(()) => true,
        Err(e) if e.raw_os_error() == Some(1314) => {
            eprintln!("skipped: symlink privilege not available on this host");
            false
        }
        Err(e) => panic!("unexpected symlink error: {e}"),
    }
}

#[test]
fn prints_symlink_target_as_written() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("target.txt"), b"x").unwrap();
    if !make_symlink(dir.path(), "link", "target.txt") {
        return;
    }

    let out = cargo_bin_cmd!("readlink")
        .arg("link")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert_eq!(text, "target.txt\n");
}

#[test]
fn no_newline_suppresses_terminator() {
    let dir = tempfile::tempdir().unwrap();
    if !make_symlink(dir.path(), "lnk", "somewhere") {
        return;
    }
    let out = cargo_bin_cmd!("readlink")
        .args(["-n", "lnk"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "somewhere");
}

#[test]
fn canonicalize_resolves_existing_paths() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("deep")).unwrap();
    let out = cargo_bin_cmd!("readlink")
        .args(["-f", "deep"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    // paths::display normalizes to forward slashes; compare on either form.
    let normalized = text.replace('/', "\\");
    // Compare against the canonicalized temp dir: CI hosts may expose
    // short (8.3) temp paths that canonicalize to the long form.
    let canonical = std::fs::canonicalize(dir.path()).unwrap().join("deep");
    let expected = canonical.to_string_lossy().replace(r"\\?\", "");
    assert_eq!(normalized, expected);
}

#[test]
fn non_link_operand_fails_with_exit_one() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("plain.txt"), b"x").unwrap();
    let out = cargo_bin_cmd!("readlink")
        .arg("plain.txt")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("readlink:"), "stderr: {err}");
}
