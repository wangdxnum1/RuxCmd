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
fn plus_extends_by_amount() {
    // GNU '+SIZE' extends BY the amount, not "at least".
    let dir = TempDir::new().unwrap();
    let p = dir.path().join("grow.bin");
    fs::write(&p, vec![0u8; 50]).unwrap();
    cargo_bin_cmd!("truncate")
        .args(["-s", "+200"])
        .arg(&p)
        .assert()
        .success();
    assert_eq!(fs::metadata(&p).unwrap().len(), 250);
    cargo_bin_cmd!("truncate")
        .args(["-s", "+10"])
        .arg(&p)
        .assert()
        .success();
    assert_eq!(fs::metadata(&p).unwrap().len(), 260);
}

#[test]
fn at_most_and_at_least_adjust() {
    let dir = TempDir::new().unwrap();
    let p = dir.path().join("bound.bin");
    fs::write(&p, vec![0u8; 50]).unwrap();
    // '>': at least — 50 stays under 100, so it grows to 100.
    cargo_bin_cmd!("truncate")
        .args(["-s", ">100"])
        .arg(&p)
        .assert()
        .success();
    assert_eq!(fs::metadata(&p).unwrap().len(), 100);
    // '<': at most — 100 is above 30, so it shrinks to 30.
    cargo_bin_cmd!("truncate")
        .args(["-s", "<30"])
        .arg(&p)
        .assert()
        .success();
    assert_eq!(fs::metadata(&p).unwrap().len(), 30);
}

#[test]
fn round_to_multiple_of_size() {
    let dir = TempDir::new().unwrap();
    let p = dir.path().join("round.bin");
    fs::write(&p, vec![0u8; 100]).unwrap();
    // '/': round down to a multiple of 7 → 98.
    cargo_bin_cmd!("truncate")
        .args(["-s", "/7"])
        .arg(&p)
        .assert()
        .success();
    assert_eq!(fs::metadata(&p).unwrap().len(), 98);
    // '%': round up to a multiple of 1000 → 1000.
    cargo_bin_cmd!("truncate")
        .args(["-s", "%1KB"])
        .arg(&p)
        .assert()
        .success();
    assert_eq!(fs::metadata(&p).unwrap().len(), 1000);
}

#[test]
fn round_to_zero_is_division_by_zero() {
    let dir = TempDir::new().unwrap();
    let p = dir.path().join("zero.bin");
    cargo_bin_cmd!("truncate")
        .args(["-s", "%0"])
        .arg(&p)
        .assert()
        .failure()
        .stderr("truncate: division by zero\n");
}

#[test]
fn io_blocks_multiplies_by_4096() {
    let dir = TempDir::new().unwrap();
    let p = dir.path().join("blocks.bin");
    cargo_bin_cmd!("truncate")
        .args(["-s", "2", "-o"])
        .arg(&p)
        .assert()
        .success();
    assert_eq!(fs::metadata(&p).unwrap().len(), 8192);
    // Suffixes compose with the block size: 1KB = 1000 bytes → 1000 blocks.
    let q = dir.path().join("blocks-kb.bin");
    cargo_bin_cmd!("truncate")
        .args(["-s", "1KB", "-o"])
        .arg(&q)
        .assert()
        .success();
    assert_eq!(fs::metadata(&q).unwrap().len(), 4_096_000);
}

#[test]
fn io_blocks_requires_size() {
    let dir = TempDir::new().unwrap();
    cargo_bin_cmd!("truncate")
        .args(["-o"])
        .arg(dir.path().join("x.bin"))
        .assert()
        .failure()
        .stderr("truncate: '--io-blocks' was specified but '--size' was not\n");
}

#[test]
fn reference_with_relative_size_adjusts() {
    let dir = TempDir::new().unwrap();
    let base = dir.path().join("base.bin");
    let p = dir.path().join("ref.bin");
    fs::write(&base, vec![0u8; 500]).unwrap();
    cargo_bin_cmd!("truncate")
        .args(["-r"])
        .arg(&base)
        .args(["-s", "+100"])
        .arg(&p)
        .assert()
        .success();
    assert_eq!(fs::metadata(&p).unwrap().len(), 600);
    // Absolute sizes with --reference are rejected like GNU.
    cargo_bin_cmd!("truncate")
        .args(["-r"])
        .arg(&base)
        .args(["-s", "100"])
        .arg(&p)
        .assert()
        .failure()
        .stderr("truncate: you must specify a relative '--size' with '--reference'\n");
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
