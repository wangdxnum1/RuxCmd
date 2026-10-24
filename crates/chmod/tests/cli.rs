//! Integration tests for chmod symbolic modes, octal regression, and the
//! junction-loop guard for -R.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_chmod")
}

fn system_cmd() -> PathBuf {
    let mut p = PathBuf::from(std::env::var_os("SystemRoot").unwrap());
    p.push("System32");
    p.push("cmd.exe");
    p
}

fn temp_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "ruxcmd-chmod-{}-{}-{}",
        tag,
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn is_readonly(path: &Path) -> bool {
    fs::metadata(path).unwrap().permissions().readonly()
}

#[test]
fn symbolic_write_bits_map_to_readonly() {
    let dir = temp_dir("sym");
    let file = dir.join("f.txt");
    fs::write(&file, "data").unwrap();

    // u-w removes the write permission -> the Windows readonly bit is set.
    let out = Command::new(bin())
        .args(["u-w"])
        .arg(&file)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(is_readonly(&file), "u-w should make the file readonly");

    // +w clears the readonly bit again.
    let out = Command::new(bin())
        .args(["+w"])
        .arg(&file)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!is_readonly(&file), "+w should make the file writable");

    // u+x has no Windows equivalent but must be accepted (exit 0, no change).
    let out = Command::new(bin())
        .args(["u+x"])
        .arg(&file)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!is_readonly(&file), "u+x must not touch the write bit");

    // Combined who lists work: go-w on a writable file -> readonly.
    let out = Command::new(bin())
        .args(["go-w"])
        .arg(&file)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(is_readonly(&file), "go-w should make the file readonly");

    // Cleanup: clear the readonly bit via chmod itself before removing the tree.
    Command::new(bin())
        .args(["644"])
        .arg(&file)
        .output()
        .unwrap();
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn octal_modes_still_work() {
    let dir = temp_dir("oct");
    let file = dir.join("f.txt");
    fs::write(&file, "data").unwrap();

    let out = Command::new(bin())
        .args(["444"])
        .arg(&file)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(is_readonly(&file));

    let out = Command::new(bin())
        .args(["644"])
        .arg(&file)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!is_readonly(&file));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn recursive_chmod_does_not_follow_junctions() {
    let dir = temp_dir("rec");
    let subdir = dir.join("a");
    fs::create_dir_all(&subdir).unwrap();
    let file = subdir.join("f.txt");
    fs::write(&file, "x").unwrap();

    let junction = subdir.join("loop");
    let out = Command::new(system_cmd())
        .args(["/C", "mklink", "/J"])
        .arg(&junction)
        .arg(&subdir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "mklink /J failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Without the symlink_metadata guard this recursion never terminates.
    let out = Command::new(bin())
        .args(["-R", "444"])
        .arg(&subdir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(is_readonly(&file));

    // Cleanup: clear the readonly bit via chmod itself before removing the tree.
    Command::new(bin())
        .args(["644"])
        .arg(&file)
        .output()
        .unwrap();
    let _ = fs::remove_dir_all(&dir);
}
