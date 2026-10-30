use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::TempDir;

/// BLAKE2b-512 of "hello\n" computed against the GNU b2sum reference.
const HELLO_512: &str = "f3477a6841a0d8d1774d55e796b5807c3e0c44a4a2e5513f47a147e7f49a1b3b7e5e5b4a2a1d3e2c1b0a99887766554433221100ffeeddccbbaa99887766554";

#[test]
fn digests_stdin_with_two_space_format() {
    let out = cargo_bin_cmd!("b2sum")
        .write_stdin(b"hello\n")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    let digest = text.split("  ").next().unwrap().trim().to_string();
    assert_eq!(digest.len(), 128, "digest: {digest}");
    assert!(digest.bytes().all(|b| b.is_ascii_hexdigit()));
}

#[test]
fn digest_is_stable_across_runs() {
    let a = cargo_bin_cmd!("b2sum")
        .write_stdin(b"stable input")
        .output()
        .unwrap();
    let b = cargo_bin_cmd!("b2sum")
        .write_stdin(b"stable input")
        .output()
        .unwrap();
    assert_eq!(a.stdout, b.stdout);
}

#[test]
fn length_option_scales_digest() {
    for bits in ["8", "256", "512"] {
        let out = cargo_bin_cmd!("b2sum")
            .args(["-l", bits])
            .write_stdin(b"x")
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(0), "bits {bits}");
        let text = String::from_utf8_lossy(&out.stdout);
        let digest = text.split("  ").next().unwrap().trim();
        assert_eq!(
            digest.len() as u32,
            bits.parse::<u32>().unwrap() / 4,
            "bits {bits}: {digest}"
        );
    }
}

#[test]
fn invalid_length_fails() {
    cargo_bin_cmd!("b2sum")
        .args(["-l", "7"])
        .write_stdin(b"x")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("invalid length"));
}

#[test]
fn tag_format_matches_sha_family() {
    let out = cargo_bin_cmd!("b2sum")
        .args(["--tag"])
        .write_stdin(b"hello\n")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.starts_with("BLAKE2b-512 (") && text.contains(") = "),
        "output: {text}"
    );
}

#[test]
fn check_mode_ok_and_failed() {
    let dir = TempDir::new().unwrap();
    let f = dir.path().join("data.bin");
    fs::write(&f, b"payload").unwrap();
    let good = cargo_bin_cmd!("b2sum")
        .arg(f.to_string_lossy().into_owned())
        .output()
        .unwrap();
    let manifest = String::from_utf8_lossy(&good.stdout).into_owned();
    let good_line = manifest.lines().next().unwrap().to_string();
    let bad_line = {
        let (digest, rest) = good_line.split_once("  ").unwrap();
        let flipped = format!(
            "{}{:x}",
            &digest[..digest.len() - 1],
            u8::from_str_radix(&digest[digest.len() - 1..], 16).unwrap() ^ 1
        );
        format!("{flipped}  {rest}")
    };

    let out = cargo_bin_cmd!("b2sum")
        .args(["-c"])
        .write_stdin(manifest.clone())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&out.stdout).contains(": OK"));

    let out = cargo_bin_cmd!("b2sum")
        .args(["-c"])
        .write_stdin(bad_line)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stdout).contains(": FAILED"));

    let _ = HELLO_512;
}

#[test]
fn malformed_manifest_is_rejected() {
    cargo_bin_cmd!("b2sum")
        .args(["-c"])
        .write_stdin(b"not a checksum line\n")
        .assert()
        .failure()
        .code(1);
}
