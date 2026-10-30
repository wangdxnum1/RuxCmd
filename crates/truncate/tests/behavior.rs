use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::TempDir;

#[test]
fn creates_and_sets_exact_size() {
    let dir = TempDir::new().unwrap();
    let p = dir.path().join("new.bin");
    cargo_bin_cmd!("truncate")
        .args(["-s", "100"])
        .arg(&p)
        .assert()
        .success();
    assert_eq!(fs::metadata(&p).unwrap().len(), 100);
}

#[test]
fn plus_extends_only() {
    let dir = TempDir::new().unwrap();
    let p = dir.path().join("grow.bin");
    fs::write(&p, vec![0u8; 50]).unwrap();
    cargo_bin_cmd!("truncate")
        .args(["-s", "+200"])
        .arg(&p)
        .assert()
        .success();
    assert_eq!(fs::metadata(&p).unwrap().len(), 200);
    cargo_bin_cmd!("truncate")
        .args(["-s", "+10"])
        .arg(&p)
        .assert()
        .success();
    assert_eq!(fs::metadata(&p).unwrap().len(), 200);
}

#[test]
fn minus_shrinks_only() {
    let dir = TempDir::new().unwrap();
    let p = dir.path().join("shrink.bin");
    fs::write(&p, vec![0u8; 200]).unwrap();
    cargo_bin_cmd!("truncate")
        .args(["-s", "-50"])
        .arg(&p)
        .assert()
        .success();
    assert_eq!(fs::metadata(&p).unwrap().len(), 150);
    cargo_bin_cmd!("truncate")
        .args(["-s", "-9999"])
        .arg(&p)
        .assert()
        .success();
    assert_eq!(fs::metadata(&p).unwrap().len(), 0);
}

#[test]
fn k_suffix_is_1024_based() {
    let dir = TempDir::new().unwrap();
    let p = dir.path().join("k.bin");
    cargo_bin_cmd!("truncate")
        .args(["-s", "2K"])
        .arg(&p)
        .assert()
        .success();
    assert_eq!(fs::metadata(&p).unwrap().len(), 2048);
}

#[test]
fn no_create_refuses_missing_files() {
    let dir = TempDir::new().unwrap();
    let p = dir.path().join("absent.bin");
    cargo_bin_cmd!("truncate")
        .args(["-c", "-s", "10"])
        .arg(&p)
        .assert()
        .failure()
        .code(1);
    assert!(!p.exists());
}

#[test]
fn missing_size_option_is_an_error() {
    let dir = TempDir::new().unwrap();
    cargo_bin_cmd!("truncate")
        .arg(dir.path().join("x.bin"))
        .assert()
        .failure()
        .code(1);
}
