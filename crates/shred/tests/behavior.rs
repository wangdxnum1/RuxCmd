use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use std::io::Read;
use tempfile::TempDir;

fn make(dir: &TempDir, name: &str, size: usize) -> String {
    let data: Vec<u8> = (0..size).map(|i| (i % 251) as u8).collect();
    let p = dir.path().join(name);
    fs::write(&p, &data).unwrap();
    p.to_string_lossy().into_owned()
}

fn run(dir: &TempDir, args: &[&str]) -> (String, String, Option<i32>) {
    let out = cargo_bin_cmd!("shred")
        .args(args)
        .current_dir(dir.path())
        .output()
        .unwrap();
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code(),
    )
}

#[test]
fn default_overwrite_zeroes_final_pass_and_keeps_file() {
    let dir = TempDir::new().unwrap();
    let p = make(&dir, "data.bin", 512);
    let (_o, _e, code) = run(&dir, &[&p]);
    assert_eq!(code, Some(0));
    let mut content = Vec::new();
    fs::File::open(&p)
        .unwrap()
        .read_to_end(&mut content)
        .unwrap();
    assert_eq!(content.len(), 512);
    assert!(content.iter().all(|&b| b == 0), "final pass must be zeros");
}

#[test]
fn single_iteration_zeroes_content() {
    let dir = TempDir::new().unwrap();
    let p = make(&dir, "one.bin", 64);
    let (_o, _e, code) = run(&dir, &["-n", "1", &p]);
    assert_eq!(code, Some(0));
    let content = fs::read(&p).unwrap();
    assert!(content.iter().all(|&b| b == 0));
}

#[test]
fn remove_option_deletes_the_file() {
    let dir = TempDir::new().unwrap();
    let p = make(&dir, "gone.bin", 32);
    let (_o, _e, code) = run(&dir, &["-u", &p]);
    assert_eq!(code, Some(0));
    assert!(!fs::exists(&p).unwrap());
}

#[test]
fn empty_file_with_remove_is_removed() {
    let dir = TempDir::new().unwrap();
    let p = make(&dir, "empty.bin", 0);
    let (_o, _e, code) = run(&dir, &["-u", &p]);
    assert_eq!(code, Some(0));
    assert!(!fs::exists(&p).unwrap());
}

#[test]
fn missing_file_exits_one() {
    let dir = TempDir::new().unwrap();
    let (_o, err, code) = run(&dir, &["no-such-file.bin"]);
    assert_eq!(code, Some(1), "shred must report failures via exit code");
    assert!(err.contains("shred: cannot open"), "stderr: {err}");
}

#[test]
fn verbose_reports_overwrite() {
    let dir = TempDir::new().unwrap();
    let p = make(&dir, "loud.bin", 16);
    let (out, _e, code) = run(&dir, &["-v", "-n", "1", &p]);
    assert_eq!(code, Some(0));
    assert!(out.contains("overwritten"), "stdout: {out}");
}
