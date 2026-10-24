//! Regression tests for P0 #6: `-c` directives must follow GNU semantics
//! (device numbers, 512-byte block counts, mount point) and human-readable
//! timestamps must be rendered in the local timezone, not UTC.

use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::TempDir;
use time::{OffsetDateTime, UtcOffset};

fn stat_cmd() -> Command {
    cargo_bin_cmd!("stat")
}

/// Runs `stat` with the given arguments and returns its stdout on success.
fn run_ok(args: &[&str]) -> String {
    let output = stat_cmd().args(args).output().unwrap();
    assert!(
        output.status.success(),
        "stat {:?} failed with {:?}: {}",
        args,
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn epoch_secs(stamp: std::io::Result<SystemTime>) -> u64 {
    stamp
        .unwrap_or(UNIX_EPOCH)
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Renders `secs` the way GNU-style human-readable timestamps are expected:
/// local timezone, `YYYY-MM-DD HH:MM:SS +HHMM`. Mirrors the implementation's
/// documented format so the test fails if `stat` renders UTC instead.
fn human_local_time(secs: i64) -> String {
    let format = time::format_description::parse_borrowed::<1>(
        "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory][offset_minute]",
    )
    .unwrap();
    let datetime = OffsetDateTime::from_unix_timestamp(secs).unwrap();
    let local = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC);
    datetime.to_offset(local).format(&format).unwrap()
}

/// Volume root of `path` (e.g. `C:\`), the documented `%m` mount-point
/// approximation on Windows.
fn volume_root(path: &Path) -> String {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().unwrap().join(path)
    };
    match absolute.components().next() {
        Some(std::path::Component::Prefix(prefix)) => match prefix.kind() {
            std::path::Prefix::Disk(letter) | std::path::Prefix::VerbatimDisk(letter) => {
                format!("{}:\\", (letter as char).to_ascii_uppercase())
            }
            std::path::Prefix::UNC(server, share)
            | std::path::Prefix::VerbatimUNC(server, share) => {
                format!(
                    "\\\\{}\\{}",
                    server.to_string_lossy(),
                    share.to_string_lossy()
                )
            }
            _ => "\\".to_string(),
        },
        _ => "\\".to_string(),
    }
}

fn make_file(dir: &TempDir, name: &str, contents: &[u8]) -> String {
    let path = dir.path().join(name);
    fs::write(&path, contents).unwrap();
    path.to_str().unwrap().to_string()
}

#[test]
fn b_counts_512_byte_blocks_and_rounds_up() {
    let dir = TempDir::new().unwrap();
    let path = make_file(&dir, "exact.bin", &[0u8; 1024]);
    assert_eq!(run_ok(&["-c", "%b", &path]).trim(), "2");

    let path = make_file(&dir, "partial.bin", &[0u8; 1025]);
    assert_eq!(run_ok(&["-c", "%b", &path]).trim(), "3");
}

#[test]
fn uppercase_y_reports_mtime_epoch_seconds() {
    let dir = TempDir::new().unwrap();
    let path = make_file(&dir, "mtime.txt", b"x");
    let expected = epoch_secs(fs::metadata(&path).unwrap().modified());
    let out = run_ok(&["-c", "%Y", &path]);
    assert_eq!(out.trim().parse::<u64>().unwrap(), expected);
}

#[test]
fn lowercase_y_renders_local_time_matching_mtime() {
    let dir = TempDir::new().unwrap();
    let path = make_file(&dir, "local.txt", b"x");
    let secs = epoch_secs(fs::metadata(&path).unwrap().modified()) as i64;
    let out = run_ok(&["-c", "%y", &path]).trim().to_string();
    assert_eq!(out, human_local_time(secs));
}

#[test]
fn t_is_device_hex_and_m_is_mount_point() {
    let dir = TempDir::new().unwrap();
    let path = make_file(&dir, "dev.txt", b"x");
    // Windows device numbers are reported as 0 (declared difference).
    assert_eq!(run_ok(&["-c", "%t", &path]).trim(), "0");
    assert_eq!(run_ok(&["-c", "%T", &path]).trim(), "0");
    // %m is the mount point (volume root), not the modification time.
    assert_eq!(
        run_ok(&["-c", "%m", &path]).trim(),
        volume_root(Path::new(&path))
    );
}

#[test]
fn unknown_directive_is_an_error() {
    let dir = TempDir::new().unwrap();
    let path = make_file(&dir, "q.txt", b"x");
    let output = stat_cmd().args(["-c", "%q", &path]).output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid directive '%q'"),
        "stderr was: {}",
        stderr
    );
    assert!(output.stdout.is_empty(), "stdout was not empty");
}

#[test]
fn directive_map_covers_gnu_names() {
    let dir = TempDir::new().unwrap();
    let path = make_file(&dir, "map.txt", &[0u8; 512]);
    let meta = fs::metadata(&path).unwrap();

    assert_eq!(run_ok(&["-c", "%s", &path]).trim(), "512");
    assert_eq!(run_ok(&["-c", "%B", &path]).trim(), "512");
    assert_eq!(run_ok(&["-c", "%n", &path]).trim(), path);
    assert_eq!(run_ok(&["-c", "%d", &path]).trim(), "0");
    assert_eq!(run_ok(&["-c", "%D", &path]).trim(), "0");
    assert_eq!(run_ok(&["-c", "%h", &path]).trim(), "1");
    assert_eq!(run_ok(&["-c", "%i", &path]).trim(), "0");
    assert_eq!(run_ok(&["-c", "%u", &path]).trim(), "0");
    assert_eq!(run_ok(&["-c", "%g", &path]).trim(), "0");
    assert_eq!(run_ok(&["-c", "%F", &path]).trim(), "regular file");
    assert_eq!(run_ok(&["-c", "%A", &path]).trim(), "-rw-rw-rw-");
    assert_eq!(run_ok(&["-c", "%a", &path]).trim(), "666");
    assert_eq!(run_ok(&["-c", "%f", &path]).trim(), "81b6");
    assert_eq!(run_ok(&["-c", "%%", &path]).trim(), "%");

    assert_eq!(
        run_ok(&["-c", "%Y", &path]).trim(),
        epoch_secs(meta.modified()).to_string()
    );
    assert_eq!(
        run_ok(&["-c", "%X", &path]).trim(),
        epoch_secs(meta.accessed()).to_string()
    );
    // Declared approximation: Windows has no change time, %Z falls back to mtime.
    assert_eq!(
        run_ok(&["-c", "%Z", &path]).trim(),
        epoch_secs(meta.modified()).to_string()
    );
    assert_eq!(
        run_ok(&["-c", "%W", &path]).trim(),
        epoch_secs(meta.created()).to_string()
    );
    assert_eq!(
        run_ok(&["-c", "%x", &path]).trim(),
        human_local_time(epoch_secs(meta.accessed()) as i64)
    );
    assert_eq!(
        run_ok(&["-c", "%z", &path]).trim(),
        human_local_time(epoch_secs(meta.modified()) as i64)
    );
}

#[test]
fn fs_mode_reports_real_block_and_space_lines() {
    let dir = TempDir::new().unwrap();
    let path = make_file(&dir, "fs.txt", b"x");
    let out = run_ok(&["-f", &path]);

    assert!(out.contains("Block size:"), "stdout was: {}", out);
    assert!(
        out.contains("Fundamental block size:"),
        "stdout was: {}",
        out
    );
    assert!(out.contains("Blocks: Total:"), "stdout was: {}", out);
    assert!(out.contains("Available:"), "stdout was: {}", out);
    assert!(
        !out.contains("unknown"),
        "stdout still has placeholders: {}",
        out
    );

    let block_size: u64 = out
        .lines()
        .find(|l| l.starts_with("Block size:"))
        .unwrap()
        .split_whitespace()
        .nth(2)
        .unwrap()
        .parse()
        .unwrap();
    assert!(block_size >= 512, "implausible block size: {}", block_size);

    let blocks_line = out
        .lines()
        .find(|l| l.starts_with("Blocks:"))
        .unwrap()
        .to_string();
    let tokens: Vec<&str> = blocks_line.split_whitespace().collect();
    let value_after = |key: &str| -> u64 {
        let index = tokens.iter().position(|t| *t == key).unwrap();
        tokens[index + 1].parse().unwrap()
    };
    let total = value_after("Total:");
    let available = value_after("Available:");
    assert!(total > 0);
    assert!(available <= total);
}
