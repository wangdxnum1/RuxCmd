use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use std::time::SystemTime;

// --- -d relative dates (GNU parse_datetime subset) ---------------------------

/// -d "1 hour ago" sets mtime one hour in the past.
#[test]
fn date_relative_one_hour_ago() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("f.txt");
    fs::write(&file, b"x").unwrap();
    cargo_bin_cmd!("touch")
        .args(["-d", "1 hour ago"])
        .arg(&file)
        .assert()
        .success();
    let mtime = fs::metadata(&file).unwrap().modified().unwrap();
    let delta = SystemTime::now().duration_since(mtime).unwrap().as_secs();
    assert!((3500..=3700).contains(&delta), "delta: {delta}s");
}

/// -d yesterday shifts by a full day.
#[test]
fn date_relative_yesterday() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("f.txt");
    fs::write(&file, b"x").unwrap();
    cargo_bin_cmd!("touch")
        .args(["-d", "yesterday"])
        .arg(&file)
        .assert()
        .success();
    let mtime = fs::metadata(&file).unwrap().modified().unwrap();
    let delta = SystemTime::now().duration_since(mtime).unwrap().as_secs();
    assert!((86000..=86800).contains(&delta), "delta: {delta}s");
}

/// -d now sets the current time.
#[test]
fn date_relative_now() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("f.txt");
    fs::write(&file, b"x").unwrap();
    cargo_bin_cmd!("touch")
        .args(["-d", "now"])
        .arg(&file)
        .assert()
        .success();
    let mtime = fs::metadata(&file).unwrap().modified().unwrap();
    let delta = SystemTime::now().duration_since(mtime).unwrap().as_secs();
    assert!(delta < 5, "delta: {delta}s");
}

/// -d "2 weeks ago" shifts by a fortnight.
#[test]
fn date_relative_two_weeks_ago() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("f.txt");
    fs::write(&file, b"x").unwrap();
    cargo_bin_cmd!("touch")
        .args(["-d", "2 weeks ago"])
        .arg(&file)
        .assert()
        .success();
    let mtime = fs::metadata(&file).unwrap().modified().unwrap();
    let delta = SystemTime::now().duration_since(mtime).unwrap().as_secs();
    assert!((1200000..=1215000).contains(&delta), "delta: {delta}s");
}
