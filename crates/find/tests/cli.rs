use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use std::time::{Duration, SystemTime};

// --- relative dates in -newermt (GNU parse_datetime subset) ------------------

/// "N unit(s) ago" shifts relative to now: a file two hours old is newer than
/// "1 hour ago" but not than "3 hours ago".
#[test]
fn newermt_relative_ago_units() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("f.txt");
    fs::write(&file, "x").unwrap();
    let two_hours_ago = SystemTime::now() - Duration::from_secs(2 * 3600);
    let times = fs::FileTimes::new().set_modified(two_hours_ago);
    fs::File::options()
        .write(true)
        .open(&file)
        .unwrap()
        .set_times(times)
        .unwrap();

    // The file is two hours old: it is newer than "3 hours ago" but older
    // than "1 hour ago".
    cargo_bin_cmd!("find")
        .arg(dir.path())
        .args(["-mindepth", "1", "-newermt", "1 hour ago"])
        .assert()
        .success()
        .stdout("");
    cargo_bin_cmd!("find")
        .arg(dir.path())
        .args(["-mindepth", "1", "-newermt", "3 hours ago"])
        .assert()
        .success()
        .stdout(predicates::str::contains("f.txt"));
}

/// yesterday/tomorrow/now shift by whole days from the current instant.
#[test]
fn newermt_named_days() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("f.txt");
    fs::write(&file, "x").unwrap();
    // A file from one year ago sits between "2 years ago" and "1 year ago".
    let one_year_ago = SystemTime::now() - Duration::from_secs(365 * 86400);
    let times = fs::FileTimes::new().set_modified(one_year_ago);
    fs::File::options()
        .write(true)
        .open(&file)
        .unwrap()
        .set_times(times)
        .unwrap();
    cargo_bin_cmd!("find")
        .arg(dir.path())
        .args(["-mindepth", "1", "-newermt", "2 years ago"])
        .assert()
        .success()
        .stdout(predicates::str::contains("f.txt"));
    cargo_bin_cmd!("find")
        .arg(dir.path())
        .args(["-mindepth", "1", "-newermt", "1 month ago"])
        .assert()
        .success()
        .stdout("");
}

/// Compact YYYYMMDD dates parse like GNU's.
#[test]
fn newermt_compact_date() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("f.txt");
    fs::write(&file, "x").unwrap();
    // 2025-06-01 12:00 UTC: clearly before both 20260101 and 20250101 local
    // midnights (UTC+8 shifts them to 2024-12-31 16:00 UTC at the latest).
    let stamp = SystemTime::UNIX_EPOCH + Duration::from_secs(1750329600);
    let times = fs::FileTimes::new().set_modified(stamp);
    fs::File::options()
        .write(true)
        .open(&file)
        .unwrap()
        .set_times(times)
        .unwrap();
    cargo_bin_cmd!("find")
        .arg(dir.path())
        .args(["-mindepth", "1", "-newermt", "20260101"])
        .assert()
        .success()
        .stdout("");
    cargo_bin_cmd!("find")
        .arg(dir.path())
        .args(["-mindepth", "1", "-newermt", "20250101"])
        .assert()
        .success()
        .stdout(predicates::str::contains("f.txt"));
}
